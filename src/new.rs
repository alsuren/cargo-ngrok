use crate::parse_code::find_handler_attrs;
use crate::parse_code::find_service_registrations;
use crate::parse_code::{find_handler_function_names, find_test_attrs};
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

fn handler_skeleton(uri: &str) -> String {
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

fn test_skeleton(uri: &str) -> String {
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

fn service_registration_skeleton(uri: &str) -> String {
    let handler_name = uri.replace(|c: char| !c.is_ascii_lowercase(), "");
    format!(".service({})", handler_name)
}

pub async fn new_test() -> Result<(), anyhow::Error> {
    let requests = crate::list::list_requests().await?;

    let trace = requests
        .into_iter()
        .filter(|t| t.response.status_code == 500)
        .next()
        .ok_or(anyhow!("no 500 responses found"))?;
    // println!("{:?}", trace.request);

    // let request_buf = base64::decode(&trace.request.raw)?;
    // println!("{:?}", std::str::from_utf8(&request_buf)?);
    // let mut request_headers = [httparse::EMPTY_HEADER; 16];
    // let mut request = httparse::Request::new(&mut request_headers);
    // let byte_count = request.parse(&request_buf)?;
    // println!("{:?} => {:?}", byte_count, request);

    let response_buf = base64::decode(&trace.response.raw)?;
    // println!("{:?}", std::str::from_utf8(&response_buf)?);
    let mut response_headers = [httparse::EMPTY_HEADER; 16];
    let mut response = httparse::Response::new(&mut response_headers);
    let byte_count = match response.parse(&response_buf)? {
        httparse::Status::Complete(byte_count) => byte_count,
        httparse::Status::Partial => anyhow::bail!("response was partial"),
    };
    // println!("{:?} => {:?}", byte_count, response);
    let response_body = std::str::from_utf8(&response_buf[byte_count..])?;

    let path = "src/main.rs";
    let content = std::fs::read_to_string(path).context(format!("reading {:?}", path))?;

    let mut lines = content
        .lines()
        .map(|s| format!("{}\n", s))
        .collect::<Vec<_>>();

    let route_path = match trace.request.uri.find('?') {
        Some(index) => trace.request.uri.get(..index).unwrap(),
        None => &trace.request.uri,
    };

    let handler_name = find_handler_function_names(&content, route_path)
        .into_iter()
        .next()
        .ok_or(anyhow!("could not find handler for {}", route_path))?;

    let existing_test = find_test_attrs(&content)
        .into_iter()
        .next()
        .ok_or(anyhow!("could not find any existing tests"))?;

    lines.insert(
        existing_test.start.line - 1,
        test_500_skeleton(&handler_name, &trace.request.uri, response_body),
    );
    std::fs::write(&path, lines.concat()).context(format!("reading {:?}", path))?;

    Ok(())
}

fn test_500_skeleton(handler_name: &str, uri: &str, response_body: &str) -> String {
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

    #[test]
    fn test_test_500_skeleton() {
        assert_eq!(
            test_500_skeleton("index", "/?param=boom", "Some error message"),
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
