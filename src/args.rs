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
    #[arg(
        long,
        conflicts_with = "pack_self",
        help = "Clear this executable's embedded configuration and exit (Linux/macOS)"
    )]
    pub truncate: bool,
    #[arg(long, help = "API authentication header value")]
    pub auth: Option<String>,
    #[arg(long, help = "Target URL")]
    pub url: Option<Url>,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub sendmail_args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sendmail_options_and_empty_recipients_are_preserved() {
        let args = Args::try_parse_from([
            "sendmail2http",
            "--url",
            "http://localhost/webhook",
            "--",
            "-i",
            "",
            "a b@example.com",
        ])
        .unwrap();
        assert_eq!(args.sendmail_args, ["-i", "", "a b@example.com"]);
        assert_eq!(args.url.unwrap().as_str(), "http://localhost/webhook");
    }

    #[test]
    fn configuration_options_after_sendmail_arguments_are_forwarded() {
        let args =
            Args::try_parse_from(["sendmail2http", "-t", "--url", "http://localhost"]).unwrap();
        assert!(args.url.is_none());
        assert_eq!(args.sendmail_args, ["-t", "--url", "http://localhost"]);
    }
}
