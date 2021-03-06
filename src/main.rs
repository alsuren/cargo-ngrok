mod list;
mod new;
mod parse_code;
mod run_ngrok;

use crate::list::{list_404, list_500};
use crate::run_ngrok::run_ngrok;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
enum CommandLine {
    Ngrok(Ngrok),
}

#[derive(StructOpt)]
/// Use ngrok traces to drive your development process.
///
/// This is heavily inspired by Dark's concept of "Trace Driven Development",
/// as described at https://darklang.github.io/docs/trace-driven-development.
#[structopt(after_help = "
The following subcommands are forwarded to ngrok for convenience:
    authtoken
    credits
    http
    start
    tcp
    tls
    update
    version
    help
")]
enum Ngrok {
    /// run your project with tracing (TODO)
    ///
    /// Starts:
    /// 1) your test runner (`cargo watch -x test`),
    /// 2) your web-server (reloading whenever the tests pass)
    /// 3) ngrok
    /// 4) your web browser, pointing at your ngrok web root.
    Develop,

    /// list requests for turning into new handlers
    ///
    /// Requests that receive 404 responses are typically ones that you haven't
    /// written handlers for yet. Use `new-handler` to define routes for these.
    _404s,

    /// list requests for turning into regression tests
    ///
    /// Requests that receive 500 responses are typically ones that caused your
    /// code to error out. Use `new-test` to make regression tests for these.
    _500s,

    /// make a new route handler from the latest 404 error
    ///
    /// Makes a skeleton route handler, plus an integration test that includes
    /// the request's payload. The integration test should initially pass (if
    /// it doesn't, please file a bug). You can then use your standard
    /// test-driven development workflow to make the request handler do what
    /// you want.
    NewHandler,

    /// make a regression test from the latest 500 error
    ///
    /// Makes an integration test that includes the request's payload.
    /// The integration test should initially fail. You can then use your
    /// standard test-driven development workflow to fix the request handler.
    NewTest,

    /// replay the latest 404 error (TODO)
    ///
    /// Use this to smoke-test the endpoint that you just wrote with
    /// `new-handler`.
    #[allow(non_camel_case_types)]
    Replay_404,

    /// replay the latest 500 error (TODO)
    ///
    /// Use this to smoke-test the endpoint that you just wrote a regression
    /// test for, with `new-test`. We've all written our share of tests that
    /// are completely disconnected from reality. You are forgiven.
    #[allow(non_camel_case_types)]
    Replay_500,

    /// The rest are passed to ngrok, for convenience.
    #[structopt(external_subcommand)]
    Other(Vec<String>),
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let CommandLine::Ngrok(options) = CommandLine::from_args();

    match options {
        Ngrok::Other(args) => {
            run_ngrok(args)?;
        }
        Ngrok::Develop => todo!(),
        Ngrok::_404s => list_404().await?.for_each(|r| println!("{}", r)),
        Ngrok::_500s => list_500().await?.for_each(|r| println!("{}", r)),
        Ngrok::NewHandler => crate::new::new_handler().await?,
        Ngrok::NewTest => crate::new::new_test().await?,
        Ngrok::Replay_404 => todo!(),
        Ngrok::Replay_500 => todo!(),
    }
    Ok(())
}
