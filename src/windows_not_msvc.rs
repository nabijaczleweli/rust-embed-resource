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

    pub fn compile_resource(&self, out_dir: &str, prefix: &str, resource: &str) {
        let out_file = format!("{}/lib{}.a", out_dir, prefix);
        let target = if env::var("TARGET").expect("No TARGET env var").starts_with("x86_64") {
            "pe-x86-64" // Default for amd64 windres
        } else {
            "pe-i386" // This is wrong for ARM Windows, but I couldn't find a triple for it (if it exists at all)
        };

        match Command::new("windres")
            .args(&["--input", resource, "--output-format=coff", "--target", target, "--output", &out_file, "--include-dir", out_dir])
            .status() {
            Ok(stat) if stat.success() => {}
            Ok(stat) => panic!("windres failed to compile \"{}\" into \"{}\" with {}", resource, out_file, stat),
            Err(e) => panic!("Couldn't to execute windres to compile \"{}\" into \"{}\": {}", resource, out_file, e),
        }
    }
}


pub fn find_windows_sdk_tool_impl(_: &str) -> Option<PathBuf> {
    None
}
