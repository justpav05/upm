// ============================================================================
// Imports
// ============================================================================
use libc::{SIGTERM, getpwuid, getuid, kill};
use std::ffi::CStr;
use std::fs;
use std::io;

use crate::types::{Error, Result};
// ============================================================================
// Utils process functions
// ============================================================================
pub fn process_exists(pid: u32) -> bool {
    let proc_path = format!("/proc/{}", pid);
    fs::metadata(proc_path).is_ok()
}

pub fn kill_process(pid: u32) -> Result<()> {
    if !process_exists(pid) {
        return Err(Error::ProcessNotFound(pid));
    }

    let result = unsafe { kill(pid as i32, SIGTERM) };
    if result != 0 {
        return Err(Error::IoError(io::Error::last_os_error()));
    }

    Ok(())
}

pub fn get_current_user() -> Result<String> {
    let uid = unsafe { getuid() };
    let pw_ptr = unsafe { getpwuid(uid) };

    if pw_ptr.is_null() {
        return Err(Error::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "User not found",
        )));
    }

    let passwd = unsafe { &*pw_ptr };
    let username = unsafe {
        CStr::from_ptr(passwd.pw_name)
            .to_string_lossy()
            .into_owned()
    };

    Ok(username)
}

pub fn is_root() -> bool {
    let uid = unsafe { getuid() };
    uid == 0
}
