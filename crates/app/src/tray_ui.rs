//! The tray icon and its menu (ADR 0067), on Windows and macOS.
//!
//! The icon needs the event loop of the thread that owns it, which is the main thread of the
//! program. [`run`] therefore blocks until the user picks "Quit"; the server runs on another thread.

use codeskimr_application::error::CodedError;
use codeskimr_cli::TrayLabels;
use codeskimr_codes::ErrorCode;
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::run_return::EventLoopExtRunReturn;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::consts::{TRAY_ICON_PIXEL, TRAY_ICON_SIZE};

/// What the user asked for through the menu.
enum Pick {
    Open,
    Log,
    Quit,
}

/// The ids of the three entries, to tell which one was picked.
struct Entries {
    open: MenuId,
    log: MenuId,
    quit: MenuId,
}

impl Entries {
    fn pick(&self, id: &MenuId) -> Option<Pick> {
        if *id == self.open {
            Some(Pick::Open)
        } else if *id == self.log {
            Some(Pick::Log)
        } else if *id == self.quit {
            Some(Pick::Quit)
        } else {
            None
        }
    }
}

fn failed<E>(_: E) -> CodedError {
    CodedError::new(ErrorCode::TrayStartFailed)
}

/// A solid square in the colour of the UI; the program has no image file of its own to load.
fn icon() -> Result<Icon, CodedError> {
    let pixels = (TRAY_ICON_SIZE * TRAY_ICON_SIZE) as usize;
    Icon::from_rgba(
        TRAY_ICON_PIXEL.repeat(pixels),
        TRAY_ICON_SIZE,
        TRAY_ICON_SIZE,
    )
    .map_err(failed)
}

fn menu(labels: &TrayLabels) -> Result<(Menu, Entries), CodedError> {
    let open = MenuItem::new(&labels.open, true, None);
    let log = MenuItem::new(&labels.log, true, None);
    let quit = MenuItem::new(&labels.quit, true, None);
    let menu = Menu::new();
    menu.append_items(&[&open, &log, &PredefinedMenuItem::separator(), &quit])
        .map_err(failed)?;
    let entries = Entries {
        open: open.id().clone(),
        log: log.id().clone(),
        quit: quit.id().clone(),
    };
    Ok((menu, entries))
}

fn show(labels: &TrayLabels, menu: Menu) -> Result<TrayIcon, CodedError> {
    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(&labels.tooltip)
        .with_icon(icon()?)
        .build()
        .map_err(failed)
}

/// Shows the tray icon and handles its menu until the user picks "Quit". `open` and `log` are
/// called on the main thread for the two other entries and must not block.
///
/// # Errors
/// `APP-603` if the icon or the menu cannot be created.
pub(crate) fn run(labels: &TrayLabels, open: impl Fn(), log: impl Fn()) -> Result<(), CodedError> {
    let (menu, entries) = menu(labels)?;
    let mut event_loop = EventLoopBuilder::<MenuEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(event);
    }));
    let mut menu = Some(menu);
    let mut icon = None;
    let mut outcome = Ok(());
    event_loop.run_return(|event, _, flow| {
        *flow = ControlFlow::Wait;
        match event {
            // The icon is created inside the loop: macOS needs the application to be running.
            Event::NewEvents(StartCause::Init) => {
                match menu.take().map(|menu| show(labels, menu)) {
                    Some(Ok(shown)) => icon = Some(shown),
                    Some(Err(error)) => {
                        outcome = Err(error);
                        *flow = ControlFlow::Exit;
                    }
                    None => {}
                }
            }
            Event::UserEvent(picked) => match entries.pick(&picked.id) {
                Some(Pick::Open) => open(),
                Some(Pick::Log) => log(),
                Some(Pick::Quit) => *flow = ControlFlow::Exit,
                None => {}
            },
            _ => {}
        }
    });
    // The icon leaves the tray when it is dropped, after the loop has ended.
    drop(icon);
    outcome
}
