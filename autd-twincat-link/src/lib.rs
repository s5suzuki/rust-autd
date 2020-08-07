#[macro_use]
extern crate lazy_static;

mod ads_error;
mod consts;
#[cfg(target_os = "windows")]
pub mod local_twincat_link;
mod native_methods;
pub mod remote_twincat_link;

#[cfg(target_os = "windows")]
pub use local_twincat_link::LocalTwinCATLink;
pub use remote_twincat_link::RemoteTwinCATLink;
