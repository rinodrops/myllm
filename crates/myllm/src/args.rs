#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Args {
    pub task: Option<String>,
    pub task_name: Option<String>,
    pub no_copy: bool,
    pub from: Option<String>,
    pub to: Option<String>,
    pub input: Option<String>,
}

impl Args {
    pub fn from_env() -> Self {
        Self::parse(std::env::args().skip(1))
    }

    pub fn parse<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut out = Self::default();
        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_ref() {
                "--task" => {
                    if let Some(value) = iter.next() {
                        out.task = Some(value.as_ref().to_string());
                    }
                }
                "--task-name" => {
                    if let Some(value) = iter.next() {
                        out.task_name = Some(value.as_ref().to_string());
                    }
                }
                "--from" => {
                    if let Some(value) = iter.next() {
                        out.from = Some(value.as_ref().to_string());
                    }
                }
                "--to" => {
                    if let Some(value) = iter.next() {
                        out.to = Some(value.as_ref().to_string());
                    }
                }
                "--no-copy" => out.no_copy = true,
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other if other.starts_with('-') => {}
                other => {
                    if out.input.is_none() {
                        out.input = Some(other.to_string());
                    }
                }
            }
        }
        out
    }

    pub fn is_single_shot(&self) -> bool {
        self.task.is_some()
    }
}

fn print_help() {
    eprintln!(
        "\
Usage: myllm [--task <id>] [--from <lang>] [--to <lang>] [--no-copy] [text]

  no arguments     stay resident (tray / hotkeys where available)
  --task <id>      run a task; `translate` uses [translation]
  --from / --to    translation language overrides
  --no-copy        do not copy the result, regardless of config
"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_task_and_langs() {
        let args = Args::parse([
            "--task",
            "translate",
            "--from",
            "en",
            "--to",
            "ja",
            "--no-copy",
            "hello",
        ]);
        assert_eq!(args.task.as_deref(), Some("translate"));
        assert_eq!(args.from.as_deref(), Some("en"));
        assert_eq!(args.to.as_deref(), Some("ja"));
        assert!(args.no_copy);
        assert_eq!(args.input.as_deref(), Some("hello"));
        assert!(args.is_single_shot());
    }

    #[test]
    fn no_args_is_resident() {
        let args = Args::parse(Vec::<&str>::new());
        assert!(!args.is_single_shot());
    }
}
