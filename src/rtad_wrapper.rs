use std::ffi::{CStr, c_char};
use std::io;

use crate::rtad;

fn check_status(status: i32) -> io::Result<()> {
    if status == rtad::rtad_error_RTAD_OK {
        return Ok(());
    }
    let message = unsafe { CStr::from_ptr(rtad::rtad_error_string(status)) };
    let kind = if status == rtad::rtad_error_RTAD_ERR_NOT_SUPPORTED {
        io::ErrorKind::Unsupported
    } else {
        io::ErrorKind::Other
    };
    Err(io::Error::new(kind, message.to_string_lossy().into_owned()))
}

pub fn validate_self_header() -> io::Result<bool> {
    let status = unsafe { rtad::rtad_validate_self_hdr() };
    if status == rtad::rtad_error_RTAD_ERR_NO_DATA {
        return Ok(false);
    }
    check_status(status)?;
    Ok(true)
}

pub fn extract_self_data() -> io::Result<Option<Vec<u8>>> {
    let mut size = 0;
    let mut data_ptr: *mut c_char = std::ptr::null_mut();
    let status = unsafe { rtad::rtad_extract_self_data(&mut data_ptr, &mut size) };
    if status == rtad::rtad_error_RTAD_ERR_NO_DATA {
        return Ok(None);
    }
    check_status(status)?;
    // RTAD_OK guarantees a non-null allocation containing size bytes.
    let bytes = unsafe { std::slice::from_raw_parts(data_ptr.cast::<u8>(), size) }.to_vec();
    unsafe { rtad::rtad_free_extracted_data(data_ptr) };
    Ok(Some(bytes))
}

/// After success, exit without calling any other RTAD self APIs.
pub fn set_self_data(data: &[u8]) -> io::Result<()> {
    check_status(unsafe { rtad::rtad_set_self_data(data.as_ptr().cast(), data.len()) })
}

/// After success, exit without calling any other RTAD self APIs.
pub fn truncate_self_data() -> io::Result<()> {
    check_status(unsafe { rtad::rtad_truncate_self_data(std::ptr::null()) })
}
