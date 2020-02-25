pub fn list_404() -> Result<Vec<String>, std::io::Error> {
    Ok(vec!["/favicon.ico".into()])
}
pub fn list_500() -> Result<Vec<String>, std::io::Error> {
    Ok(vec![])
}
