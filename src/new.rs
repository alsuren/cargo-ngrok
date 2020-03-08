use crate::list::RequestTrace;
use crate::parse_code::{
    find_handler_attr, find_handler_function_name, find_service_registration, find_test_attr,
};
use anyhow::{Context, Result};
use proc_macro2::LineColumn;

fn read_file(file_path: &str) -> Result<String> {
    std::fs::read_to_string(file_path).with_context(|| format!("reading {:?}", file_path))
}

fn write_file(file_path: &str, content: &str, edits: Vec<(LineColumn, String)>) -> Result<()> {
    let mut lines = content.lines().map(|s| format!("{}\n", s)).collect();

    for (location, code) in edits {
        insert(&mut lines, location, &code);
    }
    std::fs::write(&file_path, lines.concat()).context(format!("writing {:?}", file_path))?;

    Ok(())
}

fn insert(lines: &mut Vec<String>, location: LineColumn, code: &str) {
    lines
        .get_mut(location.line - 1)
        .unwrap()
        .insert_str(location.column, &code);
}

pub async fn new_handler() -> Result<()> {
    let trace = crate::list::latest_trace_for_code(404).await?;
    let file_path = "src/main.rs";
    let content = read_file(file_path)?;

    let edits = edits_for_new_handler(trace, &content)?;

    write_file(file_path, &content, edits)
}

fn edits_for_new_handler(trace: RequestTrace, content: &str) -> Result<Vec<(LineColumn, String)>> {
    let existing_handler = find_handler_attr(&content)?;
    let existing_test = find_test_attr(&content)?;
    let existing_service_registration = find_service_registration(&content)?;

    let safe_name = trace
        .request
        .uri
        .replace(|c: char| !c.is_ascii_lowercase(), "_");
    let handler_name = safe_name.trim_start_matches('_');

    let handler_fn = format_handler_fn(&handler_name, &trace.request.route_path());
    let integration_test = format_integration_test(&handler_name, &trace.request.uri);
    let service_registration = format!(".service({})", handler_name);

    Ok(vec![
        (existing_handler.start, handler_fn),
        (existing_test.start, integration_test),
        (existing_service_registration.end, service_registration),
    ])
}

fn format_handler_fn(handler_name: &str, route_path: &str) -> String {
    // Ignore the whitespace. Rustfmt will strip it all out.
    format!(
        r#"

#[get("{route_path}")]
async fn {handler_name}() -> impl Responder {{
    "TODO: implement this handler"
}}

"#,
        route_path = route_path,
        handler_name = handler_name,
    )
}

fn format_integration_test(handler_name: &str, uri: &str) -> String {
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

pub async fn new_test() -> Result<()> {
    let trace = crate::list::latest_trace_for_code(500).await?;

    let file_path = "src/main.rs";
    let content = read_file(file_path)?;

    let edits = edits_for_new_test(trace, &content)?;

    write_file(file_path, &content, edits)
}

fn edits_for_new_test(trace: RequestTrace, content: &str) -> Result<Vec<(LineColumn, String)>> {
    let handler_name = find_handler_function_name(&content, &trace.request.route_path())?;
    let existing_test = find_test_attr(&content)?;

    let skeleton_test = format_regression_test(
        &handler_name,
        &trace.request.uri,
        &trace.response.get_body()?,
    );
    Ok(vec![(existing_test.start, skeleton_test)])
}

fn format_regression_test(handler_name: &str, uri: &str, response_body: &str) -> String {
    let safe_name = uri.replace(|c: char| !c.is_ascii_lowercase(), "_");
    let suffix = safe_name.trim_start_matches('_');
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
    fn test_format_handler_fn() {
        assert_eq!(
            format_handler_fn("faviconico", "/favicon.ico"),
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
            format_integration_test("faviconico", "/favicon.ico"),
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
    async fn test_index_param_boom() {
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
