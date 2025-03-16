//! Types that come in handy for more than one problem

#[cfg(not(target_pointer_width = "16"))]
pub const U32_MAX: usize = u32::MAX as usize;

#[cfg(target_pointer_width = "16")]
pub const U32_MAX: usize =
    compile_error!("Cannot compile for 16-bit targets; answer would overflow");

mod bool_ext;
mod from_str_ext;

pub mod grid;

pub use bool_ext::BoolExt;
pub use from_str_ext::{TryFromStr, TryParse};
