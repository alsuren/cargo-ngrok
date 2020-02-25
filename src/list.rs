use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub uri: String,
    // TODO: think of a better way to extract the request body:
    // Currently the best I can think of is doing:
    //     pub raw: String,
    //     String::from_utf8(base64::decode(&trace.request.raw)?)?
    // and then parse the body out. Ngrok might extract it for us already.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub status_code: u32,
    // pub raw: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trace {
    pub id: String,
    pub request: Request,
    pub response: Response,
}

#[derive(Debug, Serialize, Deserialize)]
struct Traces {
    requests: Vec<Trace>,
}

pub async fn list_requests() -> Result<Vec<Trace>, anyhow::Error> {
    let traces: Traces = reqwest::get("http://127.0.0.1:4040/api/requests/http")
        .await?
        .json()
        .await?;
    Ok(traces.requests)
}

async fn list_routes_for_code(code: u32) -> Result<Vec<String>, anyhow::Error> {
    Ok(list_requests()
        .await?
        .into_iter()
        .filter(|t| t.response.status_code == code)
        .map(|t| t.request.uri)
        .collect())
}

pub async fn list_404() -> Result<Vec<String>, anyhow::Error> {
    list_routes_for_code(404).await
}

pub async fn list_500() -> Result<Vec<String>, anyhow::Error> {
    list_routes_for_code(500).await
}
