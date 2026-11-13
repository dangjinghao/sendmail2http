use std::error::Error;
use std::io;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use sendmail2http::args::Args;
use sendmail2http::sendmail::Sendmail;
use sendmail2http::{http, pack};

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = Args::parse();
    if let Some(extracted_args) = pack::extract_args()? {
        let saved =
            Args::try_parse_from(std::iter::once("sendmail2http".to_owned()).chain(extracted_args))
                .map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid embedded configuration")
                })?;
        if saved.url.is_none() || saved.pack_self || !saved.sendmail_args.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid embedded configuration",
            )
            .into());
        }
        // Explicit options override embedded defaults without duplicate clap arguments.
        args.url = args.url.or(saved.url);
        args.auth = args.auth.or(saved.auth);
    }

    let Some(url) = args.url else {
        Args::command()
            .error(
                clap::error::ErrorKind::MissingRequiredArgument,
                "--url <URL> is required unless configuration is embedded",
            )
            .exit();
    };
    if args.pack_self {
        let mut full_args = vec!["--url".to_owned(), url.to_string()];
        if let Some(auth_value) = args.auth {
            full_args.push("--auth".to_owned());
            full_args.push(auth_value);
        }
        pack::save_self(&full_args)?;
        println!("Updated this executable's embedded configuration.");
        // rtad_set_self_data replaces the file; the old process must now exit.
        return Ok(());
    }

    let sendmail_data = Sendmail::new(&args.sendmail_args)?;
    http::send_request(
        url.as_str(),
        args.auth.as_deref(),
        &sendmail_data.content,
        &sendmail_data.args,
    )?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
