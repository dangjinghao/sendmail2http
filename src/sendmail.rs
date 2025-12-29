use shell_escape::escape;
pub struct Sendmail {
    pub content: String,
    pub args: String,
}

impl Sendmail {
    pub fn new(args: &Vec<String>) -> Self {
        // we need to escape args with spaces rather than using join directly
        let escaped_args = args
            .iter()
            .map(|arg| escape(arg.into()))
            .collect::<Vec<_>>()
            .join(" ");
        // reading from stdin
        let content = std::io::read_to_string(std::io::stdin()).expect("Failed to read stdin");
        Self {
            content,
            args: escaped_args,
        }
    }
}
