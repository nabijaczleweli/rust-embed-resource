use std::process::Command;

pub struct ResourceCompiler;

impl ResourceCompiler {
    pub fn new() -> Option<Self> {
        Some(ResourceCompiler { })
    }

    pub fn compile_resource(&self, out_dir: &str, prefix: &str, resource: &str) {
        Command::new("windres")
            .args(&["--input", resource, "--output-format=coff", "--output"])
            .arg(&format!("{}/lib{}.a", out_dir, prefix))
            .status()
            .expect("Failed to execute windres");
    }
}
