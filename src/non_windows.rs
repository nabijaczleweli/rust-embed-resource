use std::env;
use std::process::Command;

pub struct ResourceCompiler {
    windres: &'static str,
}

impl ResourceCompiler {
    pub fn new() -> Option<Self> {
        match &*env::var("TARGET").unwrap() {
            "x86_64-pc-windows-gnu" => Some("x86_64-w64-mingw32-windres"),
            "i686-pc-windows-gnu" => Some("i686-w64-mingw32-windres"),
            _ => None,
        }.map(|windres| ResourceCompiler { windres: windres })
    }

    pub fn compile_resource(&self, out_dir: &str, prefix: &str, resource: &str) {
        Command::new(self.windres)
            .args(&["--input", resource, "--output-format=coff", "--output"])
            .arg(&format!("{}/lib{}.a", out_dir, prefix))
            .status()
            .expect("Failed to execute windres");
    }
}
