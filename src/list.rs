use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    method: String,
    uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    status_code: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Trace {
    id: String,
    request: Request,
    response: Response,
}

#[derive(Debug, Serialize, Deserialize)]
struct Traces {
    requests: Vec<Trace>,
}

async fn list_code(code: u32) -> Result<Vec<String>, anyhow::Error> {
    let traces: Traces = reqwest::get("http://127.0.0.1:4040/api/requests/http")
        .await?
        .json()
        .await?;
    Ok(traces
        .requests
        .into_iter()
        .filter(|t| t.response.status_code == code)
        .map(|t| t.request.uri)
        .collect())
}

pub async fn list_404() -> Result<Vec<String>, anyhow::Error> {
    list_code(404).await
}

pub async fn list_500() -> Result<Vec<String>, anyhow::Error> {
    list_code(500).await
}
