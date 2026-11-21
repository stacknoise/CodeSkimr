//! The tray icon and its menu (ADR 0067), on Windows and macOS.
//!
//! The icon needs the event loop (winit, which opens no window here) of the thread that owns it, which is the main thread of the
//! program. [`run`] therefore blocks until the user picks "Quit"; the server runs on another thread.

use codeskimr_application::error::CodedError;
use codeskimr_cli::TrayLabels;
use codeskimr_codes::ErrorCode;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
use winit::window::WindowId;

use std::time::Instant;

use crate::consts::{TRAY_ICON_RGBA, TRAY_ICON_SIZE, TRAY_SERVER_POLL};

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

/// The tray icon: the app logo from the embedded RGBA pixels.
fn icon() -> Result<Icon, CodedError> {
    Icon::from_rgba(TRAY_ICON_RGBA.to_vec(), TRAY_ICON_SIZE, TRAY_ICON_SIZE).map_err(failed)
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

/// The event loop. On macOS the program is an accessory: no Dock icon and no menu bar of its own.
fn event_loop() -> Result<EventLoop<MenuEvent>, CodedError> {
    let mut builder = EventLoop::<MenuEvent>::with_user_event();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder.with_activation_policy(ActivationPolicy::Accessory);
    }
    builder.build().map_err(failed)
}

/// The state of the event loop: the icon, once it exists, and what the menu does.
struct Tray<'a, O, L, E> {
    labels: &'a TrayLabels,
    server_ended: E,
    menu: Option<Menu>,
    icon: Option<TrayIcon>,
    entries: Entries,
    open: O,
    log: L,
    outcome: Result<(), CodedError>,
}

impl<O: Fn(), L: Fn(), E: Fn() -> bool> ApplicationHandler<MenuEvent> for Tray<'_, O, L, E> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        // The icon is created inside the loop: macOS needs the application to be running.
        if !matches!(cause, StartCause::Init) {
            return;
        }
        match self.menu.take().map(|menu| show(self.labels, menu)) {
            Some(Ok(shown)) => self.icon = Some(shown),
            Some(Err(error)) => {
                self.outcome = Err(error);
                event_loop.exit();
            }
            None => {}
        }
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    /// Looks whether the server has ended on its own; then the icon would stand for nothing.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if (self.server_ended)() {
            event_loop.exit();
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + TRAY_SERVER_POLL));
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, picked: MenuEvent) {
        match self.entries.pick(&picked.id) {
            Some(Pick::Open) => (self.open)(),
            Some(Pick::Log) => (self.log)(),
            Some(Pick::Quit) => event_loop.exit(),
            None => {}
        }
    }
}

/// Shows the tray icon and handles its menu until the user picks "Quit". `open` and `log` are
/// called on the main thread for the two other entries and must not block. `server_ended` is asked
/// about once a second; when it says yes, the loop ends too.
///
/// # Errors
/// `APP-603` if the icon, the menu or the event loop cannot be created.
pub(crate) fn run(
    labels: &TrayLabels,
    open: impl Fn(),
    log: impl Fn(),
    server_ended: impl Fn() -> bool,
) -> Result<(), CodedError> {
    let (menu, entries) = menu(labels)?;
    let mut event_loop = event_loop()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(event);
    }));
    let mut tray = Tray {
        labels,
        server_ended,
        menu: Some(menu),
        icon: None,
        entries,
        open,
        log,
        outcome: Ok(()),
    };
    event_loop.run_app_on_demand(&mut tray).map_err(failed)?;
    // The icon leaves the tray when it is dropped, after the loop has ended.
    tray.outcome
}
