//! Platform-specific clipboard tools.

mod osascript;
mod pbcopy;
mod wl_copy;
mod xclip;
mod xsel;

pub use osascript::OsaScript;
pub use pbcopy::Pbcopy;
pub use wl_copy::WlCopy;
pub use xclip::Xclip;
pub use xsel::Xsel;

use super::tool::CopyTool;

/// Get the platform-appropriate tools in priority order.
pub fn platform_tools() -> Vec<Box<dyn CopyTool>> {
    #[cfg(target_os = "macos")]
    {
        vec![Box::new(OsaScript::new()), Box::new(Pbcopy::new())]
    }

    #[cfg(target_os = "linux")]
    {
        vec![
            Box::new(Xclip::new()),
            Box::new(Xsel::new()),
            Box::new(WlCopy::new()),
        ]
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        vec![]
    }
}
