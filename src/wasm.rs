use std::ffi::c_void;
use std::io::{self, Write};
use std::ops::{Deref, DerefMut};
use std::panic::UnwindSafe;
use std::{slice, str};

use eyre::{bail, eyre, OptionExt, Result};

use crate::{
    meta::{AdventOfCode, Day, Part, Year},
    AOC,
};

#[repr(transparent)]
struct WasmBuffer([u8]);

impl WasmBuffer {
    fn new<'p>(ptr: *const c_void, len: usize) -> &'p Self {
        // TODO: check safety
        let slice = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), len) };
        unsafe { std::mem::transmute::<&[u8], &WasmBuffer>(slice) }
    }

    fn new_mut<'p>(ptr: *mut c_void, len: usize) -> &'p mut Self {
        // TODO: check safety
        let slice = unsafe { slice::from_raw_parts_mut(ptr.cast::<u8>(), len) };
        unsafe { std::mem::transmute::<&mut [u8], &mut WasmBuffer>(slice) }
    }
}

impl Deref for WasmBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WasmBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct WasmContext<'i, 'o, 'e> {
    input: &'i WasmBuffer,
    output: &'o mut WasmBuffer,
    error: &'e mut WasmBuffer,
}

impl<'i, 'o, 'e> WasmContext<'i, 'o, 'e> {
    fn new(input: &'i WasmBuffer, output: &'o mut WasmBuffer, error: &'e mut WasmBuffer) -> Self {
        Self {
            input,
            output,
            error,
        }
    }

    fn write_err(&mut self, err: impl ToString) -> io::Result<()> {
        self.error.as_mut().write_all(err.to_string().as_bytes())
    }

    fn write_res(&mut self, res: impl ToString) -> io::Result<()> {
        self.output.as_mut().write_all(res.to_string().as_bytes())
    }

    fn run<F: FnOnce(&str) -> Result<String> + UnwindSafe>(&mut self, f: F) -> u8 {
        const ERROR: u8 = 1;
        const SUCCESS: u8 = 0;

        macro_rules! bail {
            ($e:expr) => {{
                let _ = self.write_err($e);
                return ERROR;
            }};
        }
        let input = match str::from_utf8(self.input) {
            Ok(s) => s,
            Err(e) => bail!(e),
        };

        match std::panic::catch_unwind(move || (f)(input)) {
            Ok(Ok(output)) => match self.write_res(output) {
                Ok(()) => SUCCESS,
                Err(e) => bail!(e),
            },
            Ok(Err(e)) => bail!(e),
            Err(_) => bail!("panicked"),
        }
    }
}

pub unsafe extern "C" fn solve(
    year_u16: u16,
    day_u8: u8,
    part_u8: u8,
    input_ptr: *const c_void,
    input_len: usize,
    out_ptr: *mut c_void,
    out_len: usize,
    err_ptr: *mut c_void,
    err_len: usize,
) -> u8 {
    let input = WasmBuffer::new(input_ptr, input_len);
    let output = WasmBuffer::new_mut(out_ptr, out_len);
    let error = WasmBuffer::new_mut(err_ptr, err_len);

    let mut ctx = WasmContext::new(input, output, error);

    ctx.run(move |input| {
        let year = year_u16.try_into()?;
        let day = day_u8.try_into()?;
        let part = part_u8.try_into()?;

        AOC.year(year)
            .ok_or_else(|| eyre!("Haven't solved anything from {year}"))?
            .day(day)
            .ok_or_else(|| eyre!("Haven't gotten to day {day} of {year} yet"))?
            .part(part)
            .ok_or_else(|| eyre!("Haven't solved part {part} of day {day} of {year} yet"))?
            .solve(input)
    })
}
