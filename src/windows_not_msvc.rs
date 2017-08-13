use std::process::Command;

pub const SUPPORTED: bool = true;

pub fn compile_resource(out_dir: &str, prefix: &str, resource: &str) {
    if !Command::new("windres")
        .args(&["--input", resource, "--output-format=coff", "--output"])
        .arg(&format!("{}/lib{}.a", out_dir, prefix))
        .status()
        .expect("Failed to execute windres")
        .success() {
        panic!("windres failed to compile specified resource file");
    }
}
