use std::path::{PathBuf, Path};
use std::process::Command;
use winreg::enums::*;
use std::{env, fs};
use winreg;

macro_rules! try_opt {
    ($opt:expr) => {
        if let Some(o) = $opt {
            o
        } else {
            return None;
        }
    }
}

pub const SUPPORTED: bool = true;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Arch {
    X86,
    X64,
}

pub fn compile_resource(out_dir: &str, prefix: &str, resource: &str) {
    // `.res`es are linkable under MSVC as well as normal libraries.
    if !Command::new(find_windows_sdk_rc_exe().as_ref().map_or(Path::new("rc.exe"), Path::new))
        .args(&["/fo", &format!("{}/{}.lib", out_dir, prefix), resource])
        .status()
        .expect("Are you sure you have RC.EXE in your $PATH?")
        .success() {
        panic!("RC.EXE failed to compile specified resource file");
    }
}

fn find_windows_sdk_rc_exe() -> Option<PathBuf> {
    let arch = if env::var("TARGET").expect("No TARGET env var").starts_with("x86_64") {
        Arch::X64
    } else {
        Arch::X86
    };

    find_windows_kits_rc_exe("KitsRoot10", arch)
        .or_else(|| find_windows_kits_rc_exe("KitsRoot81", arch))
        .or_else(|| find_windows_kits_rc_exe("KitsRoot", arch))
        .or_else(|| find_latest_windows_sdk_rc_exe(arch))
        .or_else(|| find_windows_10_kits_rc_exe("KitsRoot10", arch))
}

// Windows 8 - 10
fn find_windows_kits_rc_exe(key: &str, arch: Arch) -> Option<PathBuf> {
    winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows Kits\Installed Roots", KEY_QUERY_VALUE)
        .and_then(|reg_key| reg_key.get_value::<String, _>(key))
        .ok()
        .and_then(|root_dir| try_bin_dir(root_dir, "bin/x86", "bin/x64", arch))
        .and_then(try_rc_exe)
}

// Windows Vista - 7
fn find_latest_windows_sdk_rc_exe(arch: Arch) -> Option<PathBuf> {
    winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Microsoft SDKs\Windows", KEY_QUERY_VALUE)
        .and_then(|reg_key| reg_key.get_value::<String, _>("CurrentInstallFolder"))
        .ok()
        .and_then(|root_dir| try_bin_dir(root_dir, "Bin", "Bin/x64", arch))
        .and_then(try_rc_exe)
}

// Windows 10 with subdir support
fn find_windows_10_kits_rc_exe(key: &str, arch: Arch) -> Option<PathBuf> {
    let root_dir = try_opt!(winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows Kits\Installed Roots", KEY_QUERY_VALUE)
        .and_then(|reg_key| reg_key.get_value::<String, _>(key))
        .ok()) + "/bin";

    for entry in try_opt!(fs::read_dir(&root_dir).ok()).filter(|d| d.is_ok()).map(Result::unwrap) {
        let fname = entry.file_name().into_string();
        let ftype = entry.file_type();
        if fname.is_err() || ftype.is_err() || ftype.unwrap().is_file() {
            continue;
        }

        let fname = entry.file_name().into_string().unwrap();
        if let Some(rc) = try_bin_dir(root_dir.clone(), &format!("{}/x86", fname), &format!("{}/x64", fname), arch).and_then(try_rc_exe) {
            return Some(rc);
        }
    }

    None
}

fn try_bin_dir(root_dir: String, x86_bin: &str, x64_bin: &str, arch: Arch) -> Option<PathBuf> {
    let mut p = PathBuf::from(root_dir);
    match arch {
        Arch::X86 => p.push(x86_bin),
        Arch::X64 => p.push(x64_bin),
    }
    if p.is_dir() { Some(p) } else { None }
}

fn try_rc_exe(mut pb: PathBuf) -> Option<PathBuf> {
    pb.push("rc.exe");
    if pb.exists() { Some(pb) } else { None }
}
