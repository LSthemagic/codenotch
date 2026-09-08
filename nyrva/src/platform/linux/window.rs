use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, KeyButMask};

fn button1_pressed(mask: KeyButMask) -> bool {
    mask.contains(KeyButMask::BUTTON1)
}

pub fn left_button_down() -> bool {
    let Ok((conn, screen_num)) = x11rb::connect(None) else {
        return false;
    };
    let Some(screen) = conn.setup().roots.get(screen_num) else {
        return false;
    };
    let Ok(cookie) = conn.query_pointer(screen.root) else {
        return false;
    };
    let Ok(reply) = cookie.reply() else {
        return false;
    };
    button1_pressed(reply.mask)
}

pub fn apply_noactivate(_app: &tauri::AppHandle) {}

pub fn attach_parent_console() {}

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
