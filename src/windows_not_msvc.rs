use std::process::Command;


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

    pub fn compile_resource(&self, out_dir: &str, prefix: &str, resource: &str) {
        let out_file = format!("{}/lib{}.a", out_dir, prefix);
        match Command::new("windres").args(&["--input", resource, "--output-format=coff", "--output", &out_file]).status() {
            Ok(stat) if stat.success() => {}
            Ok(stat) => panic!("windres failed to compile \"{}\" into \"{}\" with {}", resource, out_file, stat),
            Err(e) => panic!("Couldn't to execute windres to compile \"{}\" into \"{}\": {}", resource, out_file, e),
        }
    }
}
