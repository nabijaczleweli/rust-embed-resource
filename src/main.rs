//! Thin wrapper around `embed_resource::compile()`.


extern crate embed_resource;


use std::env;


fn main() {
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    env::set_var("TARGET",
                 if cfg!(target_arch = "x86_64") {
                     "x86_64"
                 } else {
                     "irrelevant"
                 });

    env::set_var("OUT_DIR", ".");
    embed_resource::compile(env::args().nth(1).expect("Specify the resource file to be compiled as the first argument."))
}
