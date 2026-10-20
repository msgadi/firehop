//! Security policy for the account webviews: where they may navigate, which
//! links leave the app, and which browser permissions are granted.
//!
//! Pure functions with no side effects.

use url::Url;

/// The page every account session loads.
pub const WHATSAPP_WEB: &str = "https://web.whatsapp.com/";

/// Hosts an account webview may navigate to in place. Everything else opens in
/// the user's browser.
fn is_whatsapp_host(host: &str) -> bool {
    const ROOTS: [&str; 2] = ["whatsapp.com", "whatsapp.net"];
    ROOTS.iter().any(|root| {
        host == *root
            || host
                .strip_suffix(root)
                .is_some_and(|sub| sub.ends_with('.'))
    })
}

/// Navigation inside an account webview: allowed only for WhatsApp's own HTTPS
/// pages and for in-page `blob:` and `about:` documents.
pub fn may_navigate_in_app(url: &Url) -> bool {
    match url.scheme() {
        "https" => url.host_str().is_some_and(is_whatsapp_host),
        "blob" | "about" => true,
        _ => false,
    }
}

/// Links the user clicked that should open in the system browser. Anything
/// that is not a plain web or mail link is dropped, so a chat message cannot
/// launch local programs through custom URL schemes.
pub fn may_open_externally(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http" | "mailto")
}

/// Browser capabilities WhatsApp Web asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Microphone,
    Camera,
    Notifications,
    ClipboardRead,
    Other,
}

/// Calls need the microphone and camera, chats need notifications and paste.
/// Everything else falls back to WebView2's own prompt.
pub fn grants(capability: Capability) -> bool {
    !matches!(capability, Capability::Other)
}
