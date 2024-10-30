//! Thin wrapper around `embed_resource::compile()`.


extern crate embed_resource;


use std::env;


fn main() {
    env::set_var("TARGET",
                 if cfg!(target_arch = "x86_64") {
                     "x86_64"
                 } else if cfg!(target_arch = "aarch64") {
                     "aarch64"
                 } else {
                     "irrelevant"
                 });
    #[cfg(target_os = "windows")]
    env::set_var("HOST", env::var_os("TARGET").unwrap());

    env::set_var("OUT_DIR", ".");

    let resource = env::args().nth(1).expect("Specify the resource file to be compiled as the first argument.");
    embed_resource::compile(&resource, &["VERSION=\"0.5.0\""]).manifest_required().unwrap();
    embed_resource::compile_for(&resource, &["embed_resource", "embed_resource-installer"], embed_resource::NONE).manifest_required().unwrap();
}
