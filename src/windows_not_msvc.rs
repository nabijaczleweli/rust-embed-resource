use std::process::Command;
use std::path::Path;


pub const SUPPORTED: bool = true;

pub fn compile_resource(out_dir: &str, prefix: &str, resource: &str) {
    Command::new("windres")
        .args(&["--input", resource, "--output-format=coff", "--output"])
        .arg(&format!("{}/{}.res", out_dir, prefix))
        .status()
        .unwrap();

    Command::new("ar")
        .args(&["crs", &format!("lib{}.a", prefix), &format!("{}.res", prefix)])
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();
}
