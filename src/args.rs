use std::path::PathBuf;

use clap::Parser;
use reqwest::Url;
#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about,
    long_about = "A sendmail client for sending emails via HTTP API."
)]
pub struct Args {
    #[arg(
        long = "pack",
        value_name = "PATH",
        help = "Packing current program and arguments into a new executable file"
    )]
    pub pack_path: Option<PathBuf>,
    #[arg(
        long,
        long = "truncate",
        value_name = "PATH",
        help = "Truncate saved arguments and save the truncated executable file to PATH"
    )]
    pub truncate_target_path: Option<PathBuf>,
    #[arg(long, help = "API authentication header value")]
    pub auth: String,
    #[arg(long, help = "Target URL")]
    pub url: Url,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub sendmail_args: Vec<String>,
}
