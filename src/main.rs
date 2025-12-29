use clap::Parser;
use sendmail2http::args::Args;
use sendmail2http::sendmail::Sendmail;
use sendmail2http::{http, pack};

fn main() {
    if pack::is_packed() {
        todo!();
    }
    let args = Args::parse();
    if args.pack_path.is_some() {
        todo!();
    }
    let sendmail_data = Sendmail::new(&args.sendmail_args);
    let result = http::send_request(
        args.url.as_str(),
        &args.auth,
        &sendmail_data.content,
        &sendmail_data.args,
    );
    if let Err(request_error) = result {
        eprintln!("Error sending request: {}", request_error);
    }
}
