use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
enum CommandLine {
    Ngrok(Ngrok),
}

#[derive(StructOpt)]
/// ngrok-related helpers, for trace-driven development.
///
/// The following subcommands are forwarded to ngrok:
/// authtoken, credits, http, start, tcp, tls, update, version, help.
enum Ngrok {
    #[structopt(external_subcommand)]
    Other(Vec<String>),
}

fn main() -> Result<(), std::io::Error> {
    let options = CommandLine::from_args();

    match options {
        CommandLine::Ngrok(Ngrok::Other(args)) => {
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
            let result = Command::new("ngrok").args(&args).status()?;
            println!("Done {}", result);
        }
    }
    Ok(())
}
