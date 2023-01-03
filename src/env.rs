pub fn get_cdn_origin() -> String {
    let key = "CDN_ORIGIN";

    std::env::var(key).expect(&format!("{key} is not defined"))
}
