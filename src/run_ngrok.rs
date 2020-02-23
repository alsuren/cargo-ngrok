use std::process::{Command, ExitStatus};

pub(crate) fn run_ngrok(args: Vec<String>) -> Result<ExitStatus, std::io::Error> {
    assert!([
        "authtoken",
        "credits",
        "http",
        "start",
        "tcp",
        "tls",
        "update",
        "version",
        "help",
    ]
    .contains(&args[0].as_str()));
    // TODO: work out why running ngrok under cargo-watch causes it to not
    // print anything.
    Command::new("ngrok").args(&args).status()
}
