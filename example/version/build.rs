fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if !cfg!(disable_windows_resources) && target_os == "windows" {
        embed_resource::compile("version.rc", embed_resource::NONE);
    }
}
