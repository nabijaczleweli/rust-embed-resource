fn main() {
    embed_resource::compile("version.rc", embed_resource::NONE).manifest_required().unwrap();
}
