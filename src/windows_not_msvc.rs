use std::process::Command;
use std::path::PathBuf;
use std::borrow::Cow;
use std::ffi::OsStr;
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

        // Under some msys2 environments, $MINGW_CHOST has the correct target for
        // GNU windres or llvm-windres (clang32, clang64, or clangarm64)
        let target = env::var_os("MINGW_CHOST").map(Cow::Owned).unwrap_or_else(|| {
            OsStr::new(if env::var("TARGET").expect("No TARGET env var").starts_with("x86_64") {
                    "pe-x86-64" // Default for amd64 windres
                } else {
                    "pe-i386" // This is wrong for ARM Windows, but I couldn't find a triple for it (if it exists at all)
                })
                .into()
        });

        match Command::new("windres")
            .args(&["--input", resource, "--output-format=coff", "--target"])
            .arg(target)
            .args(&["--output", &out_file, "--include-dir", out_dir])
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
