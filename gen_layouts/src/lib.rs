// See build script - Provides:
// ENTER_KEYCODE: u16
// TAB_KEYCODE: u16
// SHIFT_MODIFIER: u16
// RIGHT_ALT_MODIFIER: u16
// RIGHT_CTRL_MODIFIER: u16
// struct Layout {
//     pub layout_name: &'static str,
//     pub shift_mask: u16,
//     pub alt_mask: Option<u16>,
//     pub ctrl_mask: Option<u16>,
//     pub keycode_mask: u16,
//     pub keycodes: Box<[u16]>
// }
// LAYOUT_MAP: HashMap<&'static str, Layout>
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
