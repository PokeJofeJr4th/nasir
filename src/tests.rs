use crate::get_link_destination;

#[test]
fn urls() {
    assert_eq!(get_link_destination("https://docs.rs/releases/2", "/releases/3".into()), "https://docs.rs/releases/3".into());
}