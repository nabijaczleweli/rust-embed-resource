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

    let include_dirs = match env::args().nth(2) {
        Some(param) => vec![param],
        None => vec![],
    };
    let parameters = embed_resource::ParamsMacrosAndIncludeDirs(&["VERSION=\"0.5.0\""], include_dirs.as_slice());
    embed_resource::compile(&resource, parameters).manifest_required().unwrap();

    // Use ParamsIncludeDirs to explicitly mark Path as include directory
    embed_resource::compile_for(&resource, &["embed_resource", "embed_resource-installer"], embed_resource::ParamsIncludeDirs(include_dirs.as_slice()))
        .manifest_required()
        .unwrap();
}
