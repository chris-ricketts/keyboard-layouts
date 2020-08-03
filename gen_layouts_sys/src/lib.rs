#[cfg(not(feature = "generate"))]
mod generated;
mod types;

#[cfg(not(feature = "generate"))]
pub use generated::*;
pub use types::*;

// See build script - Provides:
// ENTER_KEYCODE: u16
// TAB_KEYCODE: u16
// SHIFT_MODIFIER: u16
// RIGHT_ALT_MODIFIER: u16
// RIGHT_CTRL_MODIFIER: u16
// LAYOUT_MAP: HashMap<&'static str, Layout>
#[cfg(feature = "generate")]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
