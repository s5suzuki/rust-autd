#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use self::windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use self::linux::*;

#[cfg(target_os = "macos")]
mod macosx;
#[cfg(target_os = "macos")]
use self::macosx::*;

mod timer;

pub use timer::Timer;
