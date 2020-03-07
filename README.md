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
    404s           list requests for turning into new handlers
    500s           list requests for turning into regression tests
    develop        run your project with tracing (TODO)
    help           Prints this message or the help of the given subcommand(s)
    new-handler    make a new route handler from the latest 404 error
    new-test       make a regression test from the latest 500 error
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

## Hacking

Progress towards MVP can be tracked at 
https://github.com/alsuren/cargo-ngrok/projects/1

I'm still trying to decide what to do after MVP. If you have any suggestions, please comment on https://github.com/alsuren/cargo-ngrok/issues/2 or poke me on gitter. Alternatively, just hack your ideas up and send me patches. I'm reasonably open to the idea of giving people merge permissions if they're enthusiastic about the project.

In my current development workflow, I have the following tabs open:

1. `~/src/actix/examples/template_yarte$ cargo watch -x test -x run`
    1. Run the tests whenever anything changes
    1. When they pass, bring up the webserver until we next make a change (it's possible to do this with less downtime if you need).
1. `$ ngrok http 8080`
    1. Bring up the tunnel, and point it at the webserver.
1. `$ curl https://xyz.ngrok.io/whatever` to cause a 404 error.
1. `~/src/cargo-ngrok$ cargo watch -x help -x fmt -x test -x build`
    1. watch the cargo-ngrok repo and test/build it
    1. `-x help` adds a block of grey text, so you can tell where one set of compiler output stops and the next starts.
    1. tests must pass before I make a new cargo-ngrok debug binary (this is useful for the below step).
1. `~/src/actix/examples/template_yarte$ watchexec  --no-ignore --verbose --watch ~/src/cargo-ngrok/target/debug/cargo-ngrok -- "git stash && cargo ngrok new-handler && cargo fmt && GIT_PAGER=cat git diff && cargo test"`
    1. whenever I have a new cargo-ngrok build (from the previous step):
        1. clean the working directory
        1. make a new skeleton `/whatever`
        1. show the diff and run the tests.
        1. Note that this only works because I have symlinked ~/src/cargo-ngrok/target/debug/cargo-ngrok into my $PATH.

I find that this allows me to prototype pretty quickly. I have also been trying out `cargo-fixeq`, which is why some of my tests jump through hoops to let the second `assert_eq!()` argument be a string literal.
