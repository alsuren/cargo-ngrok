mod run_ngrok;

use crate::run_ngrok::run_ngrok;
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
    /// run your project with tracing (TODO)
    ///
    /// Starts:
    /// 1) your test runner (`cargo watch -x test`),
    /// 2) your web-server (reloading whenever the tests pass)
    /// 3) ngrok
    /// 4) your web browser, pointing at your ngrok web root.
    Develop,

    /// list requests that returned 404 errors (TODO)
    ///
    /// These requests are typically ones that you haven't written handlers for
    /// yet. They can be used for `makeview`.
    List404s,

    /// make a new route handler from the latest 404 error (TODO)
    ///
    /// Makes a skeleton route handler, plus an integration test that includes
    /// the request's payload. The integration test should initially pass (if
    /// it doesn't, please file a bug). You can then use your standard
    /// test-driven development workflow to make the request handler do what
    /// you want.
    MakeView,

    /// list requests that returned 500 errors (TODO)
    ///
    /// These requests are typically ones that caused your code to error out.
    /// They can be used for `maketest`.
    List500s,

    /// make a regression test from the latest 500 error (TODO)
    ///
    /// Makes an integration test that includes the request's payload.
    /// The integration test should initially fail. You can then use your
    /// standard test-driven development workflow to fix the request handler.
    MakeTest,
    #[structopt(external_subcommand)]
    Other(Vec<String>),
}

fn main() -> Result<(), std::io::Error> {
    let CommandLine::Ngrok(options) = CommandLine::from_args();

    match options {
        Ngrok::Other(args) => {
            run_ngrok(args)?;
        }
        Ngrok::Develop => todo!(),
        Ngrok::List404s => todo!(),
        Ngrok::MakeView => todo!(),
        Ngrok::List500s => todo!(),
        Ngrok::MakeTest => todo!(),
    }
    Ok(())
}
