pub fn get_string(url: &str) -> Option<String> {
    get(url)?.into_string().ok()
}

pub fn get_json(url: &str) -> Option<serde_json::Value> {
    get(url)?.into_json::<serde_json::Value>().ok()
}

pub fn get_binary(url: &str) -> Option<Vec<u8>> {
    let mut reader = get(url)?.into_reader();
    let mut buf: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buf).ok();
    Some(buf)
}

fn get(url: &str) -> Option<ureq::Response> {
    let agent = ureq::builder()
        .timeout_connect(std::time::Duration::from_secs(5))
        .build();

    agent.get(url).call().ok()
}
