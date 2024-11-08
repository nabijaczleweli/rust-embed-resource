use std::path::{PathBuf, MAIN_SEPARATOR};
use self::super::apply_macros;
use std::process::Command;
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

    pub fn compile_resource<Ms: AsRef<OsStr>, Mi: IntoIterator<Item = Ms>>(&self, out_dir: &str, prefix: &str, resource: &str, macros: Mi) -> String {
        let out_file = format!("{}{}lib{}.a", out_dir, MAIN_SEPARATOR, prefix);

        // Under some msys2 environments, $MINGW_CHOST has the correct target for
        // GNU windres or llvm-windres (clang32, clang64, or clangarm64)
        let target = env::var_os("MINGW_CHOST").map(Cow::Owned).unwrap_or_else(|| {
            OsStr::new(match env::var("TARGET").expect("No TARGET env var").as_bytes() {
                    [b'x', b'8', b'6', b'_', b'6', b'4', ..] => "pe-x86-64", // "x86_64"
                    [b'a', b'a', b'r', b'c', b'h', b'6', b'4', ..] => "pe-aarch64-little", // "aarch64"
                    // windres has "pe-aarch64-little" in the strings but doesn't actually accept it on my machine,
                    // llvm-windres only has i686 and amd64; still unported
                    _ => "pe-i386",
                })
                .into()
        });

        match apply_macros(Command::new("windres")
                               .args(&["--input", resource, "--output-format=coff", "--target"])
                               .arg(target)
                               .args(&["--output", &out_file, "--include-dir", out_dir]),
                           "-D",
                           macros)
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
