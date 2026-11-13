pub struct Sendmail {
    pub content: String,
    pub args: String,
}

impl Sendmail {
    pub fn new(args: &[String]) -> std::io::Result<Self> {
        let escaped_args = shlex::try_join(args.iter().map(String::as_str))
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
        let content = std::io::read_to_string(std::io::stdin())?;
        Ok(Self {
            content,
            args: escaped_args,
        })
    }
}
