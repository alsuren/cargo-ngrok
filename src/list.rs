use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub uri: String,
    // TODO: write a .get_body() function
    // let request_buf = base64::decode(&trace.request.raw)?;
    // println!("{:?}", std::str::from_utf8(&request_buf)?);
    // let mut request_headers = [httparse::EMPTY_HEADER; 16];
    // let mut request = httparse::Request::new(&mut request_headers);
    // let byte_count = request.parse(&request_buf)?;
    // println!("{:?} => {:?}", byte_count, request);
    pub raw: String,
}

impl Request {
    pub fn route_path(&self) -> &str {
        match self.uri.find('?') {
            Some(index) => self.uri.get(..index).unwrap(),
            None => &self.uri,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub status_code: u32,
    pub raw: String,
}

impl Response {
    pub fn get_body(&self) -> anyhow::Result<String> {
        let response_buf = base64::decode(&self.raw)?;
        // println!("{:?}", std::str::from_utf8(&response_buf)?);
        let mut response_headers = [httparse::EMPTY_HEADER; 16];
        let mut response = httparse::Response::new(&mut response_headers);
        let byte_count = match response.parse(&response_buf)? {
            httparse::Status::Complete(byte_count) => byte_count,
            httparse::Status::Partial => anyhow::bail!("response was partial"),
        };
        // println!("{:?} => {:?}", byte_count, response);
        let response_body = std::str::from_utf8(&response_buf[byte_count..])?;
        Ok(response_body.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestTrace {
    pub id: String,
    pub request: Request,
    pub response: Response,
}

#[derive(Debug, Serialize, Deserialize)]
struct NgrokResponse {
    requests: Vec<RequestTrace>,
}

async fn list_requests() -> Result<Vec<RequestTrace>, anyhow::Error> {
    let resp: NgrokResponse = reqwest::get("http://127.0.0.1:4040/api/requests/http")
        .await?
        .json()
        .await?;
    Ok(resp.requests)
}

async fn traces_for_code(code: u32) -> Result<impl Iterator<Item = RequestTrace>, anyhow::Error> {
    Ok(list_requests()
        .await?
        .into_iter()
        .filter(move |t| t.response.status_code == code))
}

pub async fn latest_trace_for_code(code: u32) -> Result<RequestTrace, anyhow::Error> {
    traces_for_code(code)
        .await?
        .into_iter()
        .next()
        .ok_or(anyhow::anyhow!("no traces found for code {}", code))
}

async fn list_routes_for_code(code: u32) -> Result<impl Iterator<Item = String>, anyhow::Error> {
    Ok(traces_for_code(code)
        .await?
        .into_iter()
        .map(|t| t.request.uri))
}

pub async fn list_404() -> Result<impl Iterator<Item = String>, anyhow::Error> {
    list_routes_for_code(404).await
}

pub async fn list_500() -> Result<impl Iterator<Item = String>, anyhow::Error> {
    list_routes_for_code(500).await
}
