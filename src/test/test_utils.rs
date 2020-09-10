/*
 * The Shadow Simulator
 * See LICENSE for licensing information
 */

//! Utilities helpful for writing Rust integration tests.

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ShadowPassing {
    Yes,
    No,
}

pub struct ShadowTest<T, E> {
    name: String,
    func: Box<dyn Fn() -> Result<T, E>>,
    shadow_passing: ShadowPassing,
}

impl<T, E> fmt::Debug for ShadowTest<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShadowTest")
            .field("name", &self.name)
            .field("shadow_passing", &self.shadow_passing)
            .finish()
    }
}

impl<T, E> ShadowTest<T, E> {
    pub fn new(
        name: &str,
        func: impl Fn() -> Result<T, E> + 'static,
        shadow_passing: ShadowPassing,
    ) -> Self {
        Self {
            name: name.to_string(),
            func: Box::new(func),
            shadow_passing,
        }
    }

    pub fn run(&self) -> Result<T, E> {
        (self.func)()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn shadow_passing(&self) -> ShadowPassing {
        self.shadow_passing
    }
}

/// Runs provided tests until failure and outputs results to stdout.
pub fn run_tests<'a, I, T: 'a, E: 'a>(tests: I, summarize: bool) -> Result<Vec<T>, E>
where
    I: IntoIterator<Item = &'a ShadowTest<T, E>>,
    E: std::fmt::Debug + std::fmt::Display,
{
    let mut results = vec![];

    for test in tests {
        print!("Testing {}...", test.name());

        match test.run() {
            Err(failure) => {
                println!(" ✗ ({})", failure);
                if !summarize {
                    return Err(failure);
                }
            }
            Ok(result) => {
                results.push(result);
                println!(" ✓");
            }
        }
    }

    Ok(results)
}

// AsPtr and AsMutPtr traits inspired by https://stackoverflow.com/q/35885670

/// An object that can be converted to a pointer (possibly null).
pub trait AsPtr<T> {
    fn as_ptr(&self) -> *const T;
}

impl<T> AsPtr<T> for Option<T> {
    fn as_ptr(&self) -> *const T {
        match self {
            Some(ref v) => v as *const T,
            None => std::ptr::null(),
        }
    }
}

/// An object that can be converted to a mutable pointer (possibly null).
pub trait AsMutPtr<T> {
    fn as_mut_ptr(&mut self) -> *mut T;
}

impl<T> AsMutPtr<T> for Option<T> {
    fn as_mut_ptr(&mut self) -> *mut T {
        match self {
            Some(ref mut v) => v as *mut T,
            None => std::ptr::null_mut(),
        }
    }
}

/// Return the error message if the condition is false.
pub fn result_assert(cond: bool, message: &str) -> Result<(), String> {
    if cond {
        Ok(())
    } else {
        Err(message.to_string())
    }
}

/// Return a formatted error message if `a` and `b` are unequal.
pub fn result_assert_eq<T>(a: T, b: T, message: &str) -> Result<(), String>
where
    T: std::fmt::Debug + std::cmp::PartialEq,
{
    if a == b {
        Ok(())
    } else {
        Err(format!("{:?} != {:?} -- {}", a, b, message))
    }
}

/// Run the function and then close any given file descriptors, even if there was an error.
pub fn run_and_close_fds<'a, I, F, U>(fds: I, f: F) -> U
where
    I: IntoIterator<Item = &'a libc::c_int>,
    F: FnOnce() -> U,
{
    let rv = f();

    for fd in fds.into_iter() {
        let rv_close = unsafe { libc::close(*fd) };
        assert_eq!(rv_close, 0, "Could not close fd {}", fd);
    }

    rv
}

/// Get the current errno.
pub fn get_errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap()
}

/// Get the message for the given errno.
pub fn get_errno_message(errno: i32) -> String {
    let cstr;
    unsafe {
        let error_ptr = libc::strerror(errno);
        cstr = std::ffi::CStr::from_ptr(error_ptr)
    }
    cstr.to_string_lossy().into_owned()
}