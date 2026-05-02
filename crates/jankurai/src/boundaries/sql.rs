pub const DESTRUCTIVE_MARKERS: &[&str] = &[
    "drop table",
    "drop column",
    "truncate table",
    "delete from",
    "alter table",
];

pub fn destructive_marker(text: &str) -> Option<&'static str> {
    let lower = text.to_ascii_lowercase();
    DESTRUCTIVE_MARKERS
        .iter()
        .copied()
        .find(|marker| lower.contains(marker))
}
