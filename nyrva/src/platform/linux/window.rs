#[cfg(test)]
mod tests {
    use super::button1_pressed;
    use x11rb::protocol::xproto::KeyButMask;

    #[test]
    fn identifies_button1_mask() {
        assert!(button1_pressed(KeyButMask::BUTTON1));
        assert!(button1_pressed(KeyButMask::BUTTON1 | KeyButMask::SHIFT));
        assert!(!button1_pressed(KeyButMask::BUTTON2));
    }
}
