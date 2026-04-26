use self::super::{ParameterBundle, apply_parameters};
use std::path::{PathBuf, MAIN_SEPARATOR};
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
    pub fn is_supported(&mut self) -> Option<Cow<'static, str>> {
        None
    }

    pub fn compile_resource<Ms: AsRef<OsStr>, Mi: IntoIterator<Item = Ms>, Is: AsRef<OsStr>, Ii: IntoIterator<Item = Is>>(
        &self, out_dir: &str, prefix: &str, resource: &str, parameters: ParameterBundle<Ms, Mi, Is, Ii>)
        -> Result<String, Cow<'static, str>> {
        let out_file = format!("{}{}lib{}.a", out_dir, MAIN_SEPARATOR, prefix);

        let target = env::var("TARGET").expect("No TARGET env var");
        let is_gnullvm = target.ends_with("-gnullvm");
        let is_aarch64 = target.starts_with("aarch64-");

        // Under some msys2 environments, $MINGW_CHOST has the correct target for
        // GNU windres or llvm-windres (clang32, clang64, or clangarm64)
        let target = env::var_os("MINGW_CHOST").map(Cow::Owned).unwrap_or_else(|| {
            OsStr::new(match target.as_bytes() {
                    [b'x', b'8', b'6', b'_', b'6', b'4', ..] => "pe-x86-64", // "x86_64"
                    [b'a', b'a', b'r', b'c', b'h', b'6', b'4', ..] => "pe-aarch64-little", // "aarch64"
                    // windres has "pe-aarch64-little" in the strings but doesn't actually accept it on my machine,
                    // llvm-windres only has i686 and amd64; still unported
                    _ => "pe-i386",
                })
                .into()
        });

        let exe_name = if is_gnullvm && is_aarch64 {
            // https://github.com/llvm/llvm-project/issues/125371
            "llvm-rc"
        } else {
            "windres"
        };

        let mut command = Command::new(exe_name);
        if is_gnullvm && is_aarch64 {
            command.args(&["-fo", &out_file, "-I", out_dir, resource]);
        } else {
            command.args(&["--input", resource, "--output-format=coff", "--target"])
                .arg(target)
                .args(&["-c", "65001"]) // UTF-8, cf. https://github.com/nabijaczleweli/rust-embed-resource/pull/73
                .args(&["--output", &out_file, "--include-dir", out_dir]);
        }

        match apply_parameters(&mut command, "-D", "-I", parameters).status() {
            Ok(stat) if stat.success() => Ok(out_file),
            Ok(stat) => Err(format!("{} failed to compile \"{}\" into \"{}\" with {}", exe_name, resource, out_file, stat).into()),
            Err(e) => Err(format!("Couldn't to execute {} to compile \"{}\" into \"{}\": {}", exe_name, resource, out_file, e).into()),
        }
    }
}


pub fn find_windows_sdk_tool_impl(_: &str) -> Option<PathBuf> {
    None
}
