use std::process::Command;

pub const SUPPORTED: bool = true;

pub fn compile_resource(out_dir: &str, prefix: &str, resource: &str) {
    if let Ok(target) = std::env::var("TARGET") {
        let windres = match &*target {
            "x86_64-pc-windows-gnu" => Some("x86_64-w64-mingw32-windres"),
            "i686-pc-windows-gnu" => Some("i686-w64-mingw32-windres"),
            _ => None,
        };
        if let Some(windres) = windres {
            let out_file = format!("{}/lib{}.a", out_dir, prefix);
            match Command::new(windres).args(&["--input", resource, "--output-format=coff", "--output", &out_file]).status() {
                Ok(stat) if stat.success() => {}
                Ok(stat) => panic!("windres failed to compile \"{}\" into \"{}\" with {}", resource, out_file, stat),
                Err(e) => panic!("Couldn't to execute windres to compile \"{}\" into \"{}\": {}", resource, out_file, e),
            }
        }
    }
}
