use self::super::{ParameterBundle, env_target_and_rc};
use self::super::windres::*;
use std::ffi::{OsString, OsStr};
use std::path::{PathBuf, Path};
use std::borrow::Cow;
use std::mem;


#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceCompiler {
    compiler: Result<Compiler, Cow<'static, str>>,
}

impl ResourceCompiler {
    pub fn new() -> ResourceCompiler {
        ResourceCompiler { compiler: Compiler::probe() }
    }

    #[inline]
    pub fn is_supported(&mut self) -> Option<Cow<'static, str>> {
        match mem::replace(&mut self.compiler, Err("".into())) {
            Ok(c) => {
                self.compiler = Ok(c);
                None
            }
            Err(e) => Some(e),
        }
    }

    pub fn compile_resource<Ms: AsRef<OsStr>, Mi: IntoIterator<Item = Ms>, Is: AsRef<OsStr>, Ii: IntoIterator<Item = Is>>(
        self, out_dir: &str, prefix: &str, resource: &str, parameters: ParameterBundle<Ms, Mi, Is, Ii>)
        -> Result<String, Cow<'static, str>> {
        self.compiler.expect("Not supported but we got to compile_resource()?").compile(out_dir,
                                                                                        prefix,
                                                                                        format!("{}/{}.lib", out_dir, prefix),
                                                                                        resource,
                                                                                        parameters,
                                                                                        "/fo",
                                                                                        "/C",
                                                                                        "/no-preprocess",
                                                                                        |c| c)
    }
}


impl Compiler {
    pub fn probe() -> Result<Compiler, Cow<'static, str>> {
        let (target, rc) = env_target_and_rc()?;
        if let Some(rc) = rc {
            return guess_compiler_variant(rc);
        }

        if target.ends_with("-windows-gnu") || target.ends_with("-windows-gnullvm") {
            let executable = format!("{}-w64-mingw32-windres", &target[0..target.find('-').unwrap_or_default()]);
            if is_runnable(&executable) {
                Ok(Compiler {
                    tp: CompilerType::WindRes,
                    executable: OsString::from(executable).into(),
                })
            } else {
                Err(executable.into())
            }
        } else if target.ends_with("-windows-msvc") {
            if is_runnable("llvm-rc") {
                Ok(Compiler::llvm_rc(OsStr::new("llvm-rc").into()))
            } else {
                Err("llvm-rc".into())
            }
        } else {
            Err("".into())
        }
    }
}

/// -V will print the version in windres.
/// /? will print the help in LLVM-RC and Microsoft RC.EXE.
/// If combined, /? takes precedence over -V.
fn guess_compiler_variant(s: OsString) -> Result<Compiler, Cow<'static, str>> {
    match Command::new(&s).args(&["-V", "/?"]).output() {
        Ok(out) => {
            if out.stdout.starts_with(b"GNU windres") {
                Ok(Compiler {
                    executable: s.into(),
                    tp: CompilerType::WindRes,
                })
            } else if out.stdout.starts_with(b"OVERVIEW: Resource Converter") || out.stdout.starts_with(b"OVERVIEW: LLVM Resource Converter") {
                Ok(Compiler {
                    executable: s.into(),
                    tp: CompilerType::LlvmRc { has_no_preprocess: memmem::find(&out.stdout, b"no-preprocess").is_some() },
                })
            } else {
                Err(format!("Unknown RC compiler variant: {}", Path::new(&s).display()).into()) // TODO (MSRV 1.87): s.display()
            }
        }
        Err(err) => Err(format!("Couldn't execute {}: {}", Path::new(&s).display(), err).into()),
    }
}


fn is_runnable(s: &str) -> bool {
    Command::new(s).spawn().map(|mut c| c.kill()).is_ok()
}

pub fn find_windows_sdk_tool_impl(_: &str) -> Option<PathBuf> {
    None
}
