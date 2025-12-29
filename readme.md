# sendmail2http

A utility that converts sendmail protocol to HTTP requests.

## Features

- Protocol Conversion: Bridges sendmail interface to HTTP endpoints
- Embedded Configuration: Supports embedding HTTP-related CLI arguments data into a new executable binary, allowing this new program to carry its own configuration data

## Use Cases

- Redirect email notifications to HTTP webhooks
- Integrate legacy sendmail-based applications with modern HTTP APIs
- Create self-contained executables with pre-configured HTTP endpoints

## How It Works

1. Accepts email data via stdin (standard sendmail interface)
2. Converts the email data and CLI arguments into an HTTP request, and sends it to the specified HTTP endpoint
3. Can generate a new executable with embedded HTTP-related configuration for deployment, simplifying distribution and execution, or to solve the specific problem of hard-coding sendmail configuration in environments where passing CLI arguments is not feasible


## HTTP Request Content
```json
{
    "content": "<content>",
    "args": "<args>",
}
```

## Embedding Configuration

Execute this command:

```bash
sendmail2http --pack sendmail --url http://example.com/webhook --auth "Basic xxx"
```

to create a new executable `sendmail` with embedded configuration. This new executable can be run with sendmail arguments, e.g. `sendmail -i a@a.com b@b.com`, and it will use the embedded HTTP configuration.

Don't forget to set the appropriate permissions to make the new executable runnable:

```bash
chmod +x sendmail
```

## Options
```txt
Usage: sendmail2http [OPTIONS] --url <URL> [SENDMAIL_ARGS]...

Arguments:
  [SENDMAIL_ARGS]...  

Options:
      --pack <PATH>      Packing current program and arguments into a new executable file
      --truncate <PATH>  Truncate saved arguments and save the truncated executable file to PATH
      --auth <AUTH>      API authentication header value
      --url <URL>        Target URL
  -h, --help             Print help (see more with '--help')
  -V, --version          Print version
```

## Example

I need to send email notifications to a webhook URL `http://localhost:8080/webhook` with authentication `Basic xxx`. I can create a new executable with embedded configuration:

```bash
./sendmail2http --pack sendmail --url http://localhost:8080/webhook --auth "Basic xxx"
```

Set the executable permission:

```bash
chmod +x sendmail
```

Then I can use the `sendmail` executable as if it were a sendmail program:

```bash
echo yyy | ./sendmail -i a@a.com b@b.com
```

This will send the email data to the specified webhook URL, the server will receive an HTTP request as follows:

```txt
POST /webhook HTTP/1.1
Host: localhost:8080
Content-Type: application/json
Authorization: Basic xxx
Content-Length: 47
Accept: */*
X-Real-IP: [::1]

{"args":"-i a@a.com b@b.com","content":"yyy\n"}
```

Now you can parse this webhook content in your HTTP service, and put the `sendmail` executable in any environment that requires sendmail functionality.
