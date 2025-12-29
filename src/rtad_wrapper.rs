use std::path::PathBuf;

use crate::rtad;

pub fn validate_self_header() -> bool {
    unsafe { rtad::rtad_validate_self_hdr() == 0 }
}

pub fn truncate_self_data(new_path: &PathBuf) -> bool {
    let path_str = new_path.to_str().unwrap();
    let c_char_ptr = std::ffi::CString::new(path_str).unwrap();

    unsafe { rtad::rtad_truncate_self_data(c_char_ptr.as_ptr()) == 0 }
}

pub fn extract_self_data() -> Option<Vec<u8>> {
    let mut size: usize = 0;
    // create a c_charr buffer pointer
    let mut data_ptr: *mut i8 = std::ptr::null_mut();
    unsafe {
        let res =
            rtad::rtad_extract_self_data(&mut data_ptr as *mut *mut i8, &mut size as *mut usize);
        if res != 0 {
            return None;
        } else {
            let slice = std::slice::from_raw_parts(data_ptr as *const u8, size);
            let bytes = slice.to_vec();
            rtad::rtad_free_extracted_data(data_ptr);
            Some(bytes)
        }
    }
}

pub fn copy_self_with_data(data: &[u8], dest: &PathBuf) -> bool {
    let dest_str = dest.to_str().unwrap();
    let c_char_ptr = std::ffi::CString::new(dest_str).unwrap();

    unsafe {
        rtad::rtad_copy_self_with_data(c_char_ptr.as_ptr(), data.as_ptr() as *const i8, data.len())
            == 0
    }
}
