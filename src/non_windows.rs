use std::process::{Command, Stdio};
use std::path::{PathBuf, Path};
use std::borrow::Cow;
use std::{env, fs};


#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceCompiler {
    compiler: Option<Compiler>,
}

impl ResourceCompiler {
    pub fn new() -> ResourceCompiler {
        ResourceCompiler { compiler: Compiler::probe() }
    }

    #[inline]
    pub fn is_supported(&self) -> bool {
        self.compiler.is_some()
    }

    pub fn compile_resource(&self, out_dir: &str, prefix: &str, resource: &str) -> String {
        self.compiler.as_ref().expect("Not supported but we got to compile_resource()?").compile(out_dir, prefix, resource)
    }
}


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum CompilerType {
    /// LLVM-RC
    ///
    /// Requires a separate C preprocessor step on the source RC file
    LlvmRc,
    /// MinGW windres
    WindRes,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Compiler {
    tp: CompilerType,
    executable: Cow<'static, str>,
}

impl Compiler {
    pub fn probe() -> Option<Compiler> {
        let target = env::var("TARGET").ok()?;

        if let Some(rc) = env::var(&format!("RC_{}", target))
            .or_else(|_| env::var(&format!("RC_{}", target.replace('-', "_"))))
            .or_else(|_| env::var("RC"))
            .ok() {
            return Some(guess_compiler_variant(&rc));
        }

        if target.ends_with("-pc-windows-gnu") {
            let executable = format!("{}-w64-mingw32-windres", &target[0..target.find('-').unwrap_or_default()]);
            if is_runnable(&executable) {
                return Some(Compiler {
                    tp: CompilerType::WindRes,
                    executable: executable.into(),
                });
            }
        } else if target.ends_with("-pc-windows-msvc") {
            if is_runnable("llvm-rc") {
                return Some(Compiler {
                    tp: CompilerType::LlvmRc,
                    executable: "llvm-rc".into(),
                });
            }
        }

        None
    }

    pub fn compile(&self, out_dir: &str, prefix: &str, resource: &str) -> String {
        match self.tp {
            CompilerType::LlvmRc => {
                let out_file = format!("{}/{}.lib", out_dir, prefix);

                let preprocessed_path = format!("{}/{}-preprocessed.rc", out_dir, prefix);
                fs::write(&preprocessed_path,
                          cc::Build::new()
                              .define("RC_INVOKED", None)
                              .flag("-xc")
                              .file(resource)
                              .cargo_metadata(false)
                              .include(out_dir)
                              .expand())
                    .unwrap();

                try_command(Command::new(&self.executable[..])
                                .args(&["/fo", &out_file, "--", &preprocessed_path])
                                .stdin(Stdio::piped())
                                .current_dir(or_curdir(Path::new(resource).parent().expect("Resource parent nonexistent?"))),
                            Path::new(&self.executable[..]),
                            "compile",
                            &preprocessed_path,
                            &out_file);
                out_file
            }
            CompilerType::WindRes => {
                let out_file = format!("{}/lib{}.a", out_dir, prefix);
                try_command(Command::new(&self.executable[..])
                                .args(&["--input", resource, "--output-format=coff", "--output", &out_file, "--include-dir", out_dir]),
                            Path::new(&self.executable[..]),
                            "compile",
                            resource,
                            &out_file);
                out_file
            }
        }
    }
}

fn try_command(cmd: &mut Command, exec: &Path, action: &str, whom: &str, whre: &str) {
    match cmd.status() {
        Ok(stat) if stat.success() => {}
        Ok(stat) => panic!("{} failed to {} \"{}\" into \"{}\" with {}", exec.display(), action, whom, whre, stat),
        Err(e) => panic!("Couldn't execute {} to {} \"{}\" into \"{}\": {}", exec.display(), action, whom, whre, e),
    }
}

fn or_curdir(directory: &Path) -> &Path {
    if directory == Path::new("") {
        Path::new(".")
    } else {
        directory
    }
}

/// -V will print the version in windres.
/// /? will print the help in LLVM-RC and Microsoft RC.EXE.
/// If combined, /? takes precedence over -V.
fn guess_compiler_variant(s: &str) -> Compiler {
    match Command::new(s).args(&["-V", "/?"]).output() {
        Ok(out) => {
            if out.stdout.starts_with(b"GNU windres") {
                Compiler {
                    executable: s.to_string().into(),
                    tp: CompilerType::WindRes,
                }
            } else if out.stdout.starts_with(b"OVERVIEW: Resource Converter") {
                Compiler {
                    executable: s.to_string().into(),
                    tp: CompilerType::LlvmRc,
                }
            } else {
                panic!("Unknown RC compiler variant: {}", s)
            }
        }
        Err(err) => panic!("Couldn't execute {}: {}", s, err),
    }
}


fn is_runnable(s: &str) -> bool {
    Command::new(s).spawn().map(|mut c| c.kill()).is_ok()
}

pub fn find_windows_sdk_tool_impl(_: &str) -> Option<PathBuf> {
    None
}
