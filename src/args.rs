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
        long,
        help = "Replace this executable's embedded configuration and exit (Linux/macOS)"
    )]
    pub pack_self: bool,
    #[arg(long, help = "API authentication header value")]
    pub auth: Option<String>,
    #[arg(long, help = "Target URL")]
    pub url: Option<Url>,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub sendmail_args: Vec<String>,
}
