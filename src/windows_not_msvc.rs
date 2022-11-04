use std::process::Command;
use std::path::PathBuf;
use std::env;


#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceCompiler;


impl ResourceCompiler {
    #[inline(always)]
    pub fn new() -> ResourceCompiler {
        ResourceCompiler
    }

    #[inline(always)]
    pub fn is_supported(&self) -> bool {
        true
    }

    pub fn compile_resource(&self, out_dir: &str, prefix: &str, resource: &str) -> String {
        let out_file = format!("{}/lib{}.a", out_dir, prefix);
        // if running under any msys2 environment, the MINGW_CHOST variable will have the correct target for
        // gnu windres or llvm-windres (clang32, clang64 or clangarm64)
        let target = match env::var("MINGW_CHOST") {
            Ok(value) => value,
            Err(_) => {
                // fallback to the original target detection code
                if env::var("TARGET").expect("No TARGET env var").starts_with("x86_64") {
                    String::from("pe-x86-64") // Default for amd64 windres
                } else {
                    String::from("pe-i386") // This is wrong for ARM Windows, but I couldn't find a triple for it (if it exists at all)
                }                
            }
        };

        match Command::new("windres")
            .args(&["--input", resource, "--output-format=coff", "--target", &target, "--output", &out_file, "--include-dir", out_dir])
            .status() {
            Ok(stat) if stat.success() => {}
            Ok(stat) => panic!("windres failed to compile \"{}\" into \"{}\" with {}", resource, out_file, stat),
            Err(e) => panic!("Couldn't to execute windres to compile \"{}\" into \"{}\": {}", resource, out_file, e),
        }
        out_file
    }
}


pub fn find_windows_sdk_tool_impl(_: &str) -> Option<PathBuf> {
    None
}
