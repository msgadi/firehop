//! The few platform calls Tauri does not expose yet. This is the only place in
//! the crate that touches WebView2 COM directly.

use tauri::{Runtime, Webview};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    /// On screen: normal memory budget.
    Foreground,
    /// Hidden but still connected, so messages and notifications keep arriving.
    /// WebView2 trims caches and memory down to a minimum.
    Background,
}

/// Sets WebView2's memory usage target for a session. Failures are logged and
/// ignored: this is an optimisation, never a requirement.
#[cfg(windows)]
pub fn set_priority<R: Runtime>(webview: &Webview<R>, priority: Priority) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
        ICoreWebView2_19,
    };
    use windows::core::Interface;

    let level = match priority {
        Priority::Foreground => COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
        Priority::Background => COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    };
    let label = webview.label().to_owned();
    let result = webview.with_webview(move |platform| {
        // SAFETY: `with_webview` runs this on the UI thread that owns the
        // controller, and the controller is alive for the closure's duration.
        let outcome = unsafe {
            platform
                .controller()
                .CoreWebView2()
                .and_then(|core| core.cast::<ICoreWebView2_19>())
                .and_then(|core| core.SetMemoryUsageTargetLevel(level))
        };
        if let Err(err) = outcome {
            log::debug!("memory target for {label} not applied: {err}");
        }
    });
    if let Err(err) = result {
        log::debug!("could not reach webview for memory target: {err}");
    }
}

#[cfg(not(windows))]
pub fn set_priority<R: Runtime>(_webview: &Webview<R>, _priority: Priority) {}
