use crate::parse_code::find_handler_attrs;
use crate::parse_code::find_service_registrations;
use crate::parse_code::{find_handler_function_names, find_test_attrs};
use anyhow::Context;
use proc_macro2::LineColumn;

fn read_file(path: &str) -> anyhow::Result<(String, Vec<String>)> {
    let content = std::fs::read_to_string(path).context(format!("reading {:?}", path))?;
    let lines = content.lines().map(|s| format!("{}\n", s)).collect();

    Ok((content, lines))
}

fn insert(lines: &mut Vec<String>, location: LineColumn, code: &str) {
    dbg!(lines.get_mut(location.line - 1))
        .unwrap()
        .insert_str(location.column, &code);
}

pub async fn new_handler() -> Result<(), anyhow::Error> {
    let trace = crate::list::latest_trace_for_code(404).await?;

    let file_path = "src/main.rs";
    let (content, mut lines) = read_file(file_path)?;

    let existing_handler = find_handler_attrs(&content)?;
    let existing_test = find_test_attrs(&content)?;
    let existing_service_registration = find_service_registrations(&content)?;

    let skeleton_handler = format_handler(&trace.request.uri);
    let skeleton_test = format_integration_test(&trace.request.uri);
    let service_registration = format_service_registration(&trace.request.uri);

    insert(&mut lines, existing_handler.start, &skeleton_handler);
    insert(&mut lines, existing_test.start, &skeleton_test);
    insert(
        &mut lines,
        existing_service_registration.end,
        &service_registration,
    );

    std::fs::write(&file_path, lines.concat()).context(format!("writing {:?}", file_path))?;

    Ok(())
}

fn format_handler(uri: &str) -> String {
    // Ignore the whitespace. Rustfmt will strip it all out.
    format!(
        r#"

#[get("{uri}")]
async fn {handler_name}() -> impl Responder {{
    "TODO: implement this handler"
}}

"#,
        uri = uri,
        handler_name = uri.replace(|c: char| !c.is_ascii_lowercase(), "")
    )
}

fn format_integration_test(uri: &str) -> String {
    let handler_name = uri.replace(|c: char| !c.is_ascii_lowercase(), "");
    // Ignore the whitespace. Rustfmt will strip it all out.
    format!(
        r#"

    #[actix_rt::test]
    async fn test_{handler_name}() {{
        let mut app = atest::init_service(App::new().service({handler_name})).await;

        let req = atest::TestRequest::with_uri("{uri}").to_request();
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
        handler_name = handler_name,
        uri = uri,
    )
}

fn format_service_registration(uri: &str) -> String {
    let handler_name = uri.replace(|c: char| !c.is_ascii_lowercase(), "");
    format!(".service({})", handler_name)
}

pub async fn new_test() -> Result<(), anyhow::Error> {
    let trace = crate::list::latest_trace_for_code(500).await?;

    let file_path = "src/main.rs";
    let (content, mut lines) = read_file(file_path)?;

    let handler_name = find_handler_function_names(&content, &trace.request.route_path())?;
    let existing_test = find_test_attrs(&content)?;

    let skeleton_test = format_regression_test(
        &handler_name,
        &trace.request.uri,
        &trace.response.get_body()?,
    );

    lines.insert(existing_test.start.line - 1, skeleton_test);
    std::fs::write(&file_path, lines.concat()).context(format!("writing {:?}", file_path))?;

    Ok(())
}

fn format_regression_test(handler_name: &str, uri: &str, response_body: &str) -> String {
    let suffix = uri.replace(|c: char| !c.is_ascii_lowercase(), "");
    // Ignore the whitespace. Rustfmt will strip it all out.
    format!(
        r#"

    #[actix_rt::test]
    async fn test_{handler_name}_{suffix}() {{
        let mut app = atest::init_service(App::new().service({handler_name})).await;

        let req = atest::TestRequest::with_uri("{uri}").to_request();
        let resp = atest::call_service(&mut app, req).await;

        assert_eq!(resp.status(), 500);

        let bytes = atest::read_body(resp).await;
        assert_eq!(
            bytes,
            Bytes::from_static(b"{response_body}")
        );
    }}

"#,
        handler_name = handler_name,
        suffix = suffix,
        uri = uri,
        response_body = response_body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_handler() {
        assert_eq!(
            format_handler("/favicon.ico"),
            r#"

#[get("/favicon.ico")]
async fn faviconico() -> impl Responder {
    "TODO: implement this handler"
}

"#
        )
    }

    #[test]
    fn test_format_integration_test() {
        assert_eq!(
            format_integration_test("/favicon.ico"),
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

    #[test]
    fn test_format_regression_test() {
        assert_eq!(
            format_regression_test("index", "/?param=boom", "Some error message"),
            r#"

    #[actix_rt::test]
    async fn test_index_paramboom() {
        let mut app = atest::init_service(App::new().service(index)).await;

        let req = atest::TestRequest::with_uri("/?param=boom").to_request();
        let resp = atest::call_service(&mut app, req).await;

        assert_eq!(resp.status(), 500);

        let bytes = atest::read_body(resp).await;
        assert_eq!(
            bytes,
            Bytes::from_static(b"Some error message")
        );
    }

"#
        )
    }
}
