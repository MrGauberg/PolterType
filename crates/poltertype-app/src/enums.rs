//! Event-loop message enums.

use poltertype_core::engine::SwitcherEvent;
use poltertype_popup::PopupUiEvent;
use tray_icon::menu::MenuId;

#[derive(Debug, Clone)]
pub(crate) enum UserEvent {
    Menu(MenuId),
    Hotkey(u32),
    Engine(SwitcherEvent),
    /// Suggestion-tooltip interaction (click / timeout).
    Popup(PopupUiEvent),
    /// `config.toml` has been re-read — because the Settings window
    /// closed, or because the watcher saw the file change under a
    /// running app. Carried through the event loop because the hotkey
    /// grabs live there and are not `Send`; whoever sends this has
    /// already reloaded the store.
    SettingsChanged,
    /// Time to re-ask every plug-in what state it is in, so the tray
    /// reflects a change made somewhere else — from the command line,
    /// or an authority that expired on its own.
    PluginState,
}
