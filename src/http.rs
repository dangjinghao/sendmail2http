use reqwest::blocking::Client;

/// Sends a HTTP POST request to the specified URL with the given content and arguments.
/// auth: Authorization header value.
/// request body: {"content": "<content>", "args": "<args>"}, json encoded.
pub fn send_request(
    url: &str,
    auth: &str,
    content: &str,
    args: &str,
) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let body = serde_json::json!({
        "content": content,
        "args": args,
    });

    let response = client
        .post(url)
        .header("Authorization", auth)
        .json(&body)
        .send()?;

    response.error_for_status()?;

    Ok(())
}
