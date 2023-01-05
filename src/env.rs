pub const CDN_ORIGIN: &str = "CDN_ORIGIN";

pub const API_ORIGIN: &str = "API_ORIGIN";

pub fn url(origin: &str, endpoint: &str) -> String {
    let env = std::env::var(origin).expect(origin);

    format!("{}/{}", env, endpoint)
}
