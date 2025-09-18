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
    pub fn is_supported(&mut self) -> Option<Cow<'static, str>> {
        None
    }

    pub fn compile_resource<Ms, MsIter, Is, IsIter>(
        &self,
        out_dir: &str,
        prefix: &str,
        resource: &str,
        macros: MsIter,
        include_dirs: IsIter,
    ) -> Result<String, Cow<'static, str>>
    where
        Ms: AsRef<OsStr>,
        MsIter: IntoIterator<Item = Ms>,
        Is: AsRef<OsStr>,
        IsIter: IntoIterator<Item = Is>,
    {
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

        let mut cmd = Command::new("windres");
        cmd.args(["--input", resource, "--output-format=coff", "--target"]);
        cmd.arg(target);
        cmd.args(["--output", &out_file, "--include-dir", out_dir]);
        for dir in include_dirs {
            cmd.arg("--include-dir").arg(dir);
        }

        match apply_macros(&mut cmd, "-D", macros).status() {
            Ok(stat) if stat.success() => Ok(out_file),
            Ok(stat) => Err(format!("windres failed to compile \"{}\" into \"{}\" with {}", resource, out_file, stat).into()),
            Err(e) => Err(format!("Couldn't to execute windres to compile \"{}\" into \"{}\": {}", resource, out_file, e).into()),
        }
    }
}


pub fn find_windows_sdk_tool_impl(_: &str) -> Option<PathBuf> {
    None
}
