use std::path::PathBuf;
use std::process::Command;
use std::env;


#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceCompiler {
    windres: Option<&'static str>,
}


impl ResourceCompiler {
    pub fn new() -> ResourceCompiler {
        ResourceCompiler { windres: get_windres_executable() }
    }

    #[inline]
    pub fn is_supported(&self) -> bool {
        self.windres.is_some()
    }

    pub fn compile_resource(&self, out_dir: &str, prefix: &str, resource: &str) {
        let windres = self.windres.expect("Couldn't find windres for this platform");

        let out_file = format!("{}/lib{}.a", out_dir, prefix);
        match Command::new(windres).args(&["--input", resource, "--output-format=coff", "--output", &out_file]).status() {
            Ok(stat) if stat.success() => {}
            Ok(stat) => panic!("{} failed to compile \"{}\" into \"{}\" with {}", windres, resource, out_file, stat),
            Err(e) => panic!("Couldn't to execute {} to compile \"{}\" into \"{}\": {}", windres, resource, out_file, e),
        }
    }
}


fn get_windres_executable() -> Option<&'static str> {
    match &env::var("TARGET").ok()?[..] {
        "x86_64-pc-windows-gnu" => Some("x86_64-w64-mingw32-windres"),
        "i686-pc-windows-gnu" => Some("i686-w64-mingw32-windres"),
        _ => None,
    }
}


pub(crate) fn find_windows_sdk_tool(_tool: &str) -> Option<PathBuf> { None }
