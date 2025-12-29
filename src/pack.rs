use std::path::PathBuf;

use crate::rtad_wrapper;

pub fn is_packed() -> bool {
    rtad_wrapper::validate_self_header()
}

pub fn save_to(args: &[String], dest: &PathBuf) -> bool {
    if dest.exists() {
        std::fs::remove_file(&dest).unwrap();
    }
    let escaped_args = args
        .iter()
        .map(|arg| shlex::try_quote(arg).unwrap_or_else(|_| arg.clone().into()))
        .collect::<Vec<_>>()
        .join(" ");
    rtad_wrapper::copy_self_with_data(escaped_args.as_bytes(), dest)
}

pub fn extract_args() -> Option<Vec<String>> {
    let raw_data = rtad_wrapper::extract_self_data()?;
    let data_str = String::from_utf8_lossy(&raw_data);

    shlex::split(&data_str)
}

pub fn truncate_to(new_path: &PathBuf) -> bool {
    if new_path.exists() {
        std::fs::remove_file(&new_path).unwrap();
    };
    rtad_wrapper::truncate_self_data(new_path)
}
