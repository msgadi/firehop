/// Reads the unread-chat count WhatsApp Web puts in front of its page title,
/// e.g. `"(3) WhatsApp"` → 3. Returns 0 when the title carries no count.
pub fn unread_from_title(title: &str) -> u32 {
    title
        .strip_prefix('(')
        .and_then(|rest| rest.split_once(')'))
        .and_then(|(count, _)| count.trim_end_matches('+').parse().ok())
        .unwrap_or(0)
}
