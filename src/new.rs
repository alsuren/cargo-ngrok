use crate::parse_code::find_handler_attrs;
use crate::parse_code::find_service_registrations;
use crate::parse_code::find_test_attrs;
use anyhow::{anyhow, Context};

pub async fn new_handler() -> Result<(), anyhow::Error> {
    let requests = crate::list::list_requests().await?;

    let trace = requests
        .into_iter()
        .filter(|t| t.response.status_code == 404)
        .next()
        .ok_or(anyhow!("no 404 responses found"))?;
    println!("{:?}", trace.request);
    let path = "src/main.rs";
    let content = std::fs::read_to_string(path).context(format!("reading {:?}", path))?;

    let mut lines = content
        .lines()
        .map(|s| format!("{}\n", s))
        .collect::<Vec<_>>();

    let existing_handler = find_handler_attrs(&content)
        .into_iter()
        .next()
        .ok_or(anyhow!("could not find any existing handlers"))?;

    lines.insert(
        existing_handler.start.line - 1,
        handler_skeleton(&trace.request.uri),
    );

    let existing_test = find_test_attrs(&content)
        .into_iter()
        .next()
        .ok_or(anyhow!("could not find any existing tests"))?;

    lines.insert(
        existing_test.start.line - 1,
        test_skeleton(&trace.request.uri),
    );

    let existing_service_registration = find_service_registrations(&content)
        .into_iter()
        .next()
        .ok_or(anyhow!(
            "could not find any existing .service(...) registrations"
        ))?;

    lines
        .get_mut(existing_service_registration.end.line)
        .unwrap()
        .insert_str(
            existing_service_registration.end.column,
            &service_registration_skeleton(&trace.request.uri),
        );
    std::fs::write(&path, lines.concat()).context(format!("reading {:?}", path))?;

    Ok(())
}

pub async fn new_test() -> Result<(), anyhow::Error> {
    todo!()
}

fn handler_skeleton(uri: &str) -> String {
    // Ignore the whitespace. Rustfmt will strip it all out.
    format!(
        r#"

#[get("{}")]
async fn {}() -> impl Responder {{
    "TODO: implement this handler"
}}

"#,
        uri,
        uri.replace(|c: char| !c.is_ascii_lowercase(), "")
    )
}

fn test_skeleton(uri: &str) -> String {
    let handler_name = uri.replace(|c: char| !c.is_ascii_lowercase(), "");
    // Ignore the whitespace. Rustfmt will strip it all out.
    format!(
        r#"

    #[actix_rt::test]
    async fn test_{}() {{
        let mut app = atest::init_service(App::new().service({})).await;

        let req = atest::TestRequest::with_uri("{}").to_request();
        let resp = atest::call_service(&mut app, req).await;

        dbg!(resp.status());
        assert!(resp.status().is_success());

        let bytes = atest::read_body(resp).await;
        assert_eq!(
            bytes,
            Bytes::from_static(b"TODO: implement this handler")
        );
    }}

"#,
        handler_name, handler_name, uri,
    )
}

fn service_registration_skeleton(uri: &str) -> String {
    let handler_name = uri.replace(|c: char| !c.is_ascii_lowercase(), "");
    format!(".service({})", handler_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_skeleton() {
        assert_eq!(
            handler_skeleton("/favicon.ico"),
            r#"

#[get("/favicon.ico")]
async fn faviconico() -> impl Responder {
    "TODO: implement this handler"
}

"#
        )
    }

    #[test]
    fn test_test_skeleton() {
        assert_eq!(
            test_skeleton("/favicon.ico"),
            r#"

    #[actix_rt::test]
    async fn test_faviconico() {
        let mut app = atest::init_service(App::new().service(faviconico)).await;

        let req = atest::TestRequest::with_uri("/favicon.ico").to_request();
        let resp = atest::call_service(&mut app, req).await;

        dbg!(resp.status());
        assert!(resp.status().is_success());

        let bytes = atest::read_body(resp).await;
        assert_eq!(
            bytes,
            Bytes::from_static(b"TODO: implement this handler")
        );
    }

"#
        )
    }
}
