//! Types that come in handy for more than one problem

#[cfg(not(target_pointer_width = "16"))]
pub const U32_MAX: usize = u32::MAX as usize;

#[cfg(target_pointer_width = "16")]
pub const U32_MAX: usize =
    compile_error!("Cannot compile for 16-bit targets; answer would overflow");

pub mod bool_ext;
pub mod from_str_ext;
pub mod grid;
