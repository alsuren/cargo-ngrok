# cargo-ngrok
Use ngrok traces to drive your development process.

This is heavily inspired by Dark's concept of "Trace Driven Development", as described at
https://darklang.github.io/docs/trace-driven-development.

It's is only a sketch at the moment, but this is what I'm expecting the mvp
command-line to look like:
```
USAGE:
    cargo ngrok <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    404s           list requests for turning into new handlers (TODO)
    500s           list requests for turning into regression tests (TODO)
    develop        run your project with tracing (TODO)
    help           Prints this message or the help of the given subcommand(s)
    new-handler    make a new route handler from the latest 404 error (TODO)
    new-test       make a regression test from the latest 500 error (TODO)
    replay-404     replay the latest 404 error (TODO)
    replay-500     replay the latest 500 error (TODO)

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
```

The MVP will only support generating actix-web request handlers of the form:
```
#[get("/")]
async fn no_params() -> &'static str {
    "Hello world!\r\n"
}
```

Progress towards MVP can be tracked at 
https://github.com/alsuren/cargo-ngrok/projects/1
