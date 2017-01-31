use std::path::{PathBuf, Path};
use std::process::Command;
use winreg::enums::*;
use std::env;
use winreg;

pub const SUPPORTED: bool = true;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Arch {
    X86,
    X64,
}

pub fn compile_resource(out_dir: &str, prefix: &str, resource: &str) {
    let rc_path = find_windows_sdk_bin_dir().and_then(|mut x| {
        x.push("rc.exe");
        if x.exists() { Some(x) } else { None }
    });

    // `.res`es are linkable under MSVC as well as normal libraries.
    Command::new(rc_path.as_ref().map_or(Path::new("rc.exe"), Path::new))
        .args(&["/fo", &format!("{}/{}.lib", out_dir, prefix), resource])
        .status()
        .expect("Are you sure you have RC.EXE in your $PATH?");
}

fn find_windows_sdk_bin_dir() -> Option<PathBuf> {
    let arch = if env::var("TARGET").unwrap().starts_with("x86_64") {
        Arch::X64
    } else {
        Arch::X86
    };

    find_windows_kits_bin_dir("KitsRoot10", arch)
        .or_else(|| find_windows_kits_bin_dir("KitsRoot81", arch))
        .or_else(|| find_windows_kits_bin_dir("KitsRoot", arch))
        .or_else(|| find_latest_windows_sdk_bin_dir(arch))
}

// Windows 8 - 10
fn find_windows_kits_bin_dir(key: &str, arch: Arch) -> Option<PathBuf> {
    winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows Kits\Installed Roots", KEY_QUERY_VALUE)
        .and_then(|reg_key| reg_key.get_value::<String, _>(key))
        .ok()
        .and_then(|root_dir| try_bin_dir(root_dir, "bin/x86", "bin/x64", arch))
}

// Windows Vista - 7
fn find_latest_windows_sdk_bin_dir(arch: Arch) -> Option<PathBuf> {
    winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Microsoft SDKs\Windows", KEY_QUERY_VALUE)
        .and_then(|reg_key| reg_key.get_value::<String, _>("CurrentInstallFolder"))
        .ok()
        .and_then(|root_dir| try_bin_dir(root_dir, "Bin", "Bin/x64", arch))
}

fn try_bin_dir(root_dir: String, x86_bin: &str, x64_bin: &str, arch: Arch) -> Option<PathBuf> {
    let mut p = PathBuf::from(root_dir);
    match arch {
        Arch::X86 => p.push(x86_bin),
        Arch::X64 => p.push(x64_bin),
    }
    if p.is_dir() { Some(p) } else { None }
}
