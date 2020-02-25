use crate::parse_code::find_route_attrs;
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

    let existing_route = find_route_attrs(&content)
        .into_iter()
        .next()
        .ok_or(anyhow!("could not find any existing routes"))?;

    let mut lines = content
        .lines()
        .map(|s| format!("{}\n", s))
        .collect::<Vec<_>>();

    lines.insert(
        existing_route.start.line - 1,
        route_skeleton(&trace.request.uri),
    );

    std::fs::write(&path, lines.concat()).context(format!("reading {:?}", path))?;

    Ok(())
}

pub async fn new_test() -> Result<(), anyhow::Error> {
    todo!()
}

fn route_skeleton(uri: &str) -> String {
    // Ignore the whitespace. Rustfmt will strip it all out.
    format!(
        r#"

#[get("{}")]
async fn {}() -> impl Responder {{
    ""
}}

"#,
        uri,
        uri.replace(|c: char| !c.is_ascii_lowercase(), "")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_skeleton() {
        assert_eq!(
            route_skeleton("/favicon.ico"),
            r#"

#[get("/favicon.ico")]
async fn faviconico() -> impl Responder {
    ""
}

"#
        )
    }
}
