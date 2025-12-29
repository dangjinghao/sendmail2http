use std::env;

use clap::Parser;
use sendmail2http::args::Args;
use sendmail2http::sendmail::Sendmail;
use sendmail2http::{http, pack};

fn main() {
    let sendmail_data;
    let args;
    if pack::is_packed() {
        match pack::extract_args() {
            Some(extracted_args) => {
                let current_args = env::args().collect::<Vec<String>>();
                // insert extracted_args after the first argument (program name)
                let mut combined_args = vec![current_args[0].clone()];
                combined_args.extend(extracted_args.iter().cloned());
                combined_args.extend(current_args.iter().skip(1).cloned());

                args = Args::parse_from(&combined_args);
                sendmail_data = Sendmail::new(&args.sendmail_args);
            }
            None => {
                panic!("Failed to extract sendmail arguments.");
            }
        }
    } else {
        args = Args::parse();
        if args.pack_path.is_some() {
            let pack_path = args.pack_path.unwrap();
            let full_args = vec![
                "--url".to_string(),
                args.url.to_string().clone(),
                "--auth".to_string(),
                args.auth.clone(),
            ];

            if pack::save_to(&full_args, &pack_path) {
                println!(
                    "Successfully packed sendmail data into {}.",
                    &pack_path.display()
                );
            } else {
                panic!("Failed to pack sendmail data.");
            }
            return;
        }
        sendmail_data = Sendmail::new(&args.sendmail_args);
    }

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
