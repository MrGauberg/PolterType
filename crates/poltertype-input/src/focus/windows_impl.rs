//! Windows foreground-window queries: which process owns it, where it
//! is, and where its caret sits.
//!
//! `GetForegroundWindow` → `GetWindowThreadProcessId` →
//! `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` →
//! `QueryFullProcessImageNameW` → basename. Needs no special
//! permission and works across elevation levels, which is what
//! `LIMITED_INFORMATION` exists for.
//!
//! The caret comes from `GetGUIThreadInfo` on the **foreground**
//! thread, so — unlike the desktop-wide AT-SPI slot on Linux — the
//! sample belongs to the focused window by construction and carries
//! neither an age nor a pid to prove it with.

use std::cell::RefCell;
use std::path::Path;
use std::time::Duration;

use tracing::{debug, warn};
use windows::Win32::Foundation::{CloseHandle, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
use windows::Win32::UI::WindowsAndMessaging::{
    GUITHREADINFO, GWL_STYLE, GetClassNameW, GetForegroundWindow, GetGUIThreadInfo, GetWindowLongW,
    GetWindowRect, GetWindowThreadProcessId,
};
use windows::core::PWSTR;

use super::{CaretHint, FocusTracker, SensitiveInput};

const ES_PASSWORD_STYLE: u32 = 0x0020;

struct UiaClient {
    automation: IUIAutomation,
}

impl Drop for UiaClient {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

thread_local! {
    static UIA_CLIENT: RefCell<Option<UiaClient>> = const { RefCell::new(None) };
}

fn with_uia<T>(f: impl FnOnce(&IUIAutomation) -> Option<T>) -> Option<T> {
    UIA_CLIENT.with(|slot| {
        if slot.borrow().is_none() {
            let created = unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED).ok().ok()?;
                let automation: IUIAutomation =
                    CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
                Some(UiaClient { automation })
            };
            *slot.borrow_mut() = created;
        }
        let borrow = slot.borrow();
        f(&borrow.as_ref()?.automation)
    })
}

fn uia_password_state() -> Option<bool> {
    with_uia(|automation| unsafe {
        let focused = automation.GetFocusedElement().ok()?;
        Some(focused.CurrentIsPassword().ok()?.as_bool())
    })
}

fn native_password_state() -> Option<bool> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let thread = GetWindowThreadProcessId(hwnd, None);
        if thread == 0 {
            return None;
        }
        let mut gui = GUITHREADINFO {
            cbSize: u32::try_from(size_of::<GUITHREADINFO>()).ok()?,
            ..Default::default()
        };
        GetGUIThreadInfo(thread, &mut gui).ok()?;
        if gui.hwndFocus.0.is_null() {
            return None;
        }

        let mut class = [0u16; 128];
        let len = GetClassNameW(gui.hwndFocus, &mut class);
        if len <= 0 {
            return None;
        }
        let class = String::from_utf16_lossy(&class[..len as usize]).to_ascii_lowercase();
        if class != "edit" && !class.starts_with("richedit") {
            return None;
        }

        let style = GetWindowLongW(gui.hwndFocus, GWL_STYLE) as u32;
        Some(style & ES_PASSWORD_STYLE != 0)
    }
}

pub struct WindowsFocusTracker;

impl FocusTracker for WindowsFocusTracker {
    fn focused_exe(&self) -> Option<String> {
        // Safety: a chain of standard Win32 calls; we close the
        // process handle exactly once before returning.
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }
            let mut pid: u32 = 0;
            let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return None;
            }
            let process = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(h) => h,
                Err(e) => {
                    warn!(?e, pid, "OpenProcess failed");
                    return None;
                }
            };
            let mut buf = [0u16; 1024];
            let mut len: u32 = buf.len() as u32;
            let q = QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut len,
            );
            let _ = CloseHandle(process);
            if let Err(e) = q {
                warn!(?e, pid, "QueryFullProcessImageNameW failed");
                return None;
            }
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            let name = Path::new(&path).file_name()?.to_string_lossy().into_owned();
            Some(name)
        }
    }

    fn focused_window_geometry(&self) -> Option<crate::focus::FocusedWindowGeometry> {
        // Safety: `GetWindowRect` writes into the RECT we own; no
        // handles to release.
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }
            let mut rect = RECT::default();
            if let Err(e) = GetWindowRect(hwnd, &mut rect) {
                warn!(?e, "GetWindowRect failed");
                return None;
            }
            let width = u32::try_from(rect.right.saturating_sub(rect.left)).ok()?;
            let height = u32::try_from(rect.bottom.saturating_sub(rect.top)).ok()?;
            Some(crate::focus::FocusedWindowGeometry {
                x: rect.left,
                y: rect.top,
                width,
                height,
                // The caret below is read off this same foreground
                // window, so nothing ever has to be proved against it.
                pid: None,
            })
        }
    }

    fn caret_hint(&self) -> Option<CaretHint> {
        // Safety: every call writes only into stack structures we own.
        // `GetGUIThreadInfo` fails outright unless `cbSize` is set
        // first, which is the one ordering constraint here.
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }
            let thread = GetWindowThreadProcessId(hwnd, None);
            if thread == 0 {
                return None;
            }
            let mut gui = GUITHREADINFO {
                cbSize: u32::try_from(size_of::<GUITHREADINFO>()).ok()?,
                ..Default::default()
            };
            if let Err(e) = GetGUIThreadInfo(thread, &mut gui) {
                debug!(
                    ?e,
                    "GetGUIThreadInfo failed — anchoring the tooltip to the window"
                );
                return None;
            }
            if gui.hwndCaret.0.is_null() {
                debug!("focused window owns no caret — anchoring the tooltip to the window");
                return None;
            }
            // `rcCaret` is client-space in `hwndCaret`, which is often
            // a child control rather than the toplevel we measure.
            let mut origin = POINT {
                x: gui.rcCaret.left,
                y: gui.rcCaret.top,
            };
            if !ClientToScreen(gui.hwndCaret, &mut origin).as_bool() {
                debug!("ClientToScreen failed — anchoring the tooltip to the window");
                return None;
            }
            let mut window = RECT::default();
            if let Err(e) = GetWindowRect(hwnd, &mut window) {
                warn!(?e, "GetWindowRect failed");
                return None;
            }
            caret_hint_from(origin, gui.rcCaret, window)
        }
    }

    fn sensitive_input(&self) -> SensitiveInput {
        match uia_password_state().or_else(native_password_state) {
            Some(true) => SensitiveInput::Sensitive,
            Some(false) => SensitiveInput::NotSensitive,
            None => SensitiveInput::Unknown,
        }
    }

    fn backend_name(&self) -> &'static str {
        "windows-foreground-process"
    }
}

/// The window-relative hint for a caret already resolved to the screen
/// point `origin`, whose client-space rectangle is `caret`, inside the
/// foreground window `window`.
///
/// `None` for a caret of no height: a control that owns a caret it
/// never shows leaves a collapsed rectangle at the client origin
/// behind, and anchoring to that puts the tooltip in the window's
/// top-left corner rather than where anyone is typing.
fn caret_hint_from(origin: POINT, caret: RECT, window: RECT) -> Option<CaretHint> {
    let height = u32::try_from(caret.bottom.saturating_sub(caret.top)).ok()?;
    if height == 0 {
        return None;
    }
    Some(CaretHint {
        x: origin.x.saturating_sub(window.left),
        y: origin.y.saturating_sub(window.top),
        height,
        // Queried live from the foreground thread, so there is no
        // sample to go stale and no other window it could belong to.
        age: Duration::ZERO,
        pid: None,
        window: None,
    })
}

#[cfg(test)]
mod tests;
