use std::io;

use crate::rtad_wrapper;

pub fn is_packed() -> io::Result<bool> {
    rtad_wrapper::validate_self_header()
}

fn encode_args(args: &[String]) -> io::Result<String> {
    shlex::try_join(args.iter().map(String::as_str))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}

pub fn save_self(args: &[String]) -> io::Result<()> {
    rtad_wrapper::set_self_data(encode_args(args)?.as_bytes())
}

pub fn extract_args() -> io::Result<Option<Vec<String>>> {
    let Some(raw_data) = rtad_wrapper::extract_self_data()? else {
        return Ok(None);
    };
    let data_str = String::from_utf8(raw_data)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let args = shlex::split(&data_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid embedded arguments"))?;
    Ok(Some(args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_round_trips_special_arguments() {
        let args = vec![
            "--url".into(),
            "http://localhost/a?x=1&y=2".into(),
            "--auth".into(),
            "Bearer 'quoted' \"value\" \\ 中文\n".into(),
            "".into(),
        ];
        assert_eq!(shlex::split(&encode_args(&args).unwrap()).unwrap(), args);
    }

    #[test]
    fn nul_bytes_are_rejected_instead_of_silently_changed() {
        assert_eq!(
            encode_args(&["bad\0argument".into()]).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }
}
