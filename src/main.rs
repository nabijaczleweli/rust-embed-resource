//! Thin wrapper around `embed_resource::compile()`.


extern crate embed_resource;


use std::env;


fn main() {
    #[cfg(target_os = "windows")]
    env::set_var("TARGET",
                 if cfg!(target_arch = "x86_64") {
                     "x86_64"
                 } else {
                     "irrelevant"
                 });

    env::set_var("OUT_DIR", ".");

    let resource = env::args().nth(1).expect("Specify the resource file to be compiled as the first argument.");
    embed_resource::compile(&resource);
    embed_resource::compile_for(&resource, &["embed_resource", "embed_resource-installer"]);
}
