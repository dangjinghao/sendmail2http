use reqwest::blocking::Client;

/// Sends a HTTP POST request to the specified URL with the given content and arguments.
/// auth: Authorization header value.
/// request body: {"content": "<content>", "args": "<args>"}, json encoded.
pub fn send_request(
    url: &str,
    auth: Option<&str>,
    content: &str,
    args: &str,
) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let body = serde_json::json!({
        "content": content,
        "args": args,
    });
    let mut req_builder = client.post(url).json(&body);
    if let Some(auth_value) = auth {
        req_builder = req_builder.header("Authorization", auth_value);
    }
    req_builder.send()?.error_for_status()?;

    Ok(())
}
