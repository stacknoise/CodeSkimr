//! The terminal of `codeskimr level2 confirm`: where the warning is shown and the name is typed.
//!
//! In a window that the server opened (`--from-ui`, ADR 0071) the standard streams cannot be
//! trusted: the program that started it may have none (the tray copy has none), and the new
//! console then only gets what the starter passed on. So on Windows the window talks to the
//! console of its own through `CONIN$` and `CONOUT$`. Everywhere else, and for the command typed
//! in a terminal, it is standard input and output.

use std::cell::RefCell;
use std::io::{self, BufRead, IsTerminal, Write};

use codeskimr_application::error::CodedError;
use codeskimr_cli::{confirm_prompt, confirm_target, confirm_warning};
use codeskimr_codes::ErrorCode;
use codeskimr_i18n::Messages;

use crate::consts::LINE_BREAK;
use crate::level2_consent::Target;

/// Input and output of one confirmation.
pub(crate) struct Console {
    input: Box<dyn BufRead>,
    output: Box<dyn Write>,
    interactive: bool,
}

impl Console {
    /// Standard input and output; interactive only if both are a terminal.
    pub(crate) fn standard() -> Self {
        let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();
        Self {
            input: Box::new(io::stdin().lock()),
            output: Box::new(io::stdout()),
            interactive,
        }
    }

    /// The console of a window the server opened: its own console on Windows, which the copy of
    /// the program started from the tray does not have as standard streams.
    ///
    /// # Errors
    /// `SEC-403` if the window has no console to talk to; nothing is confirmed then.
    #[cfg(windows)]
    pub(crate) fn window() -> Result<Self, CodedError> {
        use std::fs::{File, OpenOptions};
        let denied = |_| CodedError::new(ErrorCode::IndexerNotPermitted);
        let input = File::open(crate::consts::CONSOLE_INPUT).map_err(denied)?;
        let output = OpenOptions::new()
            .write(true)
            .open(crate::consts::CONSOLE_OUTPUT)
            .map_err(denied)?;
        Ok(Self {
            interactive: input.is_terminal() && output.is_terminal(),
            input: Box::new(io::BufReader::new(input)),
            output: Box::new(output),
        })
    }

    /// A console over the given streams, for the tests.
    #[cfg(test)]
    pub(crate) fn over(input: Box<dyn BufRead>, output: Box<dyn Write>, interactive: bool) -> Self {
        Self {
            input,
            output,
            interactive,
        }
    }

    /// Writes `text` and a line break. A closed pipe is the reader's decision; nobody is told.
    pub(crate) fn say(&mut self, text: &str) {
        for part in [text, LINE_BREAK] {
            if self.output.write_all(part.as_bytes()).is_err() {
                return;
            }
        }
        self.output.flush().ok();
    }

    /// Reads one line; an empty answer (end of input) is not a confirmation.
    fn answer(&mut self) -> Result<String, CodedError> {
        let mut answer = String::new();
        self.input
            .read_line(&mut answer)
            .map_err(|_| CodedError::new(ErrorCode::IndexerNotPermitted))?;
        Ok(answer)
    }

    /// Waits for Enter, so that the answer in a window can be read before the window closes.
    pub(crate) fn wait_for_enter(&mut self) {
        self.answer().ok();
    }
}

/// Shows what the user is asked to confirm and reads the answer. Without a terminal nothing is
/// confirmed.
///
/// # Errors
/// `SEC-403` if the console is not a terminal or cannot be read.
pub(crate) fn ask(
    (messages, language): (&Messages, &str),
    console: &RefCell<Console>,
    target: &Target,
) -> Result<String, CodedError> {
    let mut console = console.borrow_mut();
    if !console.interactive {
        return Err(CodedError::new(ErrorCode::IndexerNotPermitted));
    }
    let say = |text: Result<String, _>| text.map_err(CodedError::caused_by);
    let target_text = confirm_target(
        messages,
        language,
        &target.name,
        &target.place,
        target.branch.as_deref(),
    );
    let prompt = confirm_prompt(messages, language, &target.name);
    for text in [confirm_warning(messages, language), target_text, prompt] {
        console.say(&say(text)?);
    }
    console.answer()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::{Arc, Mutex};

    use super::*;

    /// A writer whose text the test can read afterwards.
    #[derive(Clone, Default)]
    struct Shared(Arc<Mutex<Vec<u8>>>);

    impl Write for Shared {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn target() -> Target {
        Target {
            name: "shop".to_owned(),
            place: "/work/shop".to_owned(),
            branch: None,
        }
    }

    fn console(typed: &str, interactive: bool) -> (RefCell<Console>, Shared) {
        let shown = Shared::default();
        let console = Console::over(
            Box::new(Cursor::new(typed.as_bytes().to_vec())),
            Box::new(shown.clone()),
            interactive,
        );
        (RefCell::new(console), shown)
    }

    #[test]
    fn the_window_shows_the_warning_the_project_and_the_prompt_and_reads_the_typed_name() {
        let messages = Messages::load().unwrap();
        let (console, shown) = console("shop\n", true);
        let answer = ask((&messages, "en"), &console, &target()).unwrap();
        assert_eq!(answer.trim(), "shop");
        let text = String::from_utf8(shown.0.lock().unwrap().clone()).unwrap();
        assert!(text.contains("not sandboxed"), "{text}");
        assert!(text.contains("/work/shop"), "{text}");
        assert!(text.contains("(\"shop\")"), "{text}");
    }

    #[test]
    fn without_a_terminal_nothing_is_asked_and_nothing_is_confirmed() {
        let messages = Messages::load().unwrap();
        let (console, shown) = console("shop\n", false);
        let denied = ask((&messages, "en"), &console, &target()).unwrap_err();
        assert_eq!(
            codeskimr_codes::Coded::code(&denied),
            ErrorCode::IndexerNotPermitted
        );
        assert_eq!(*shown.0.lock().unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn waiting_for_enter_takes_one_line_and_an_empty_input_ends_it() {
        let (console, _) = console("\nrest\n", true);
        console.borrow_mut().wait_for_enter();
        assert_eq!(console.borrow_mut().answer().unwrap(), "rest\n");
        console.borrow_mut().wait_for_enter();
    }
}
