use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::AtomicBool;
use std::path::{PathBuf, Path};
use std::process::Command;
use vswhom::VsFindResult;
use winreg::enums::*;
use std::{env, fs};
use winreg;


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
        // `.res`es are linkable under MSVC as well as normal libraries.
        if !Command::new(find_windows_sdk_rc_exe().as_ref().map_or(Path::new("rc.exe"), Path::new))
            .args(&["/fo", &format!("{}/{}.lib", out_dir, prefix), resource])
            .status()
            .expect("Are you sure you have RC.EXE in your $PATH?")
            .success() {
            panic!("RC.EXE failed to compile specified resource file");
        }
    }
}


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Arch {
    X86,
    X64,
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
        .or_else(|| find_with_vswhom(arch))
}

fn find_with_vswhom(arch: Arch) -> Option<PathBuf> {
    let res = VsFindResult::search();
    res.as_ref()
        .and_then(|res| res.windows_sdk_root.as_ref())
        .map(PathBuf::from)
        .and_then(|mut root| {
            let ver = root.file_name().expect("malformed vswhom-returned SDK root").to_os_string();
            root.pop();
            root.pop();
            root.push("bin");
            root.push(ver);
            try_bin_dir(root, "x86", "x64", arch)
        })
        .and_then(try_rc_exe)
        .or_else(move || {
            res.and_then(|res| res.windows_sdk_root)
                .map(PathBuf::from)
                .and_then(|mut root| {
                    root.pop();
                    root.pop();
                    try_bin_dir(root, "bin/x86", "bin/x64", arch)
                })
                .and_then(try_rc_exe)
        })
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
    let kit_root = (winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows Kits\Installed Roots", KEY_QUERY_VALUE)
        .and_then(|reg_key| reg_key.get_value::<String, _>(key))
        .ok())?;
    include_windows_10_kits(&kit_root);
    let root_dir = kit_root + "/bin";

    for entry in fs::read_dir(&root_dir).ok()?.filter(|d| d.is_ok()).map(Result::unwrap) {
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

/// Update %INCLUDE% to contain all \Include\<version>\ folders before invoking rc.exe
/// (https://github.com/nabijaczleweli/rust-embed-resource/pull/17),
/// fixing "Unable to find windows.h" errors (https://github.com/nabijaczleweli/rust-embed-resource/issues/11)
fn include_windows_10_kits(kit_root: &str) {
    static IS_INCLUDED: AtomicBool = AtomicBool::new(false);

    if !IS_INCLUDED.swap(true, SeqCst) {
        include_windows_10_kits_impl(kit_root);
    }
}

fn include_windows_10_kits_impl(kit_root: &str) {
    const VAR_INCLUDE: &str = "INCLUDE";

    if let Ok(include_root) = fs::read_dir(kit_root.to_string() + r"\Include\") {
        let mut include = env::var(VAR_INCLUDE).unwrap_or_default();
        if !include.ends_with(';') {
            include.push(';');
        }

        get_dirs(include_root).filter_map(|dir| fs::read_dir(dir.path()).ok()).for_each(|dir| {
            get_dirs(dir).for_each(|sub_dir| if let Some(sub_dir) = sub_dir.path().to_str() {
                if !include.contains(sub_dir) {
                    include.push_str(sub_dir);
                    include.push(';');
                }
            })
        });

        env::set_var(VAR_INCLUDE, include);
    }
}

fn get_dirs(read_dir: fs::ReadDir) -> impl Iterator<Item = fs::DirEntry> {
    read_dir.filter_map(|dir| dir.ok()).filter(|dir| dir.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
}

fn try_bin_dir<R: Into<PathBuf>>(root_dir: R, x86_bin: &str, x64_bin: &str, arch: Arch) -> Option<PathBuf> {
    try_bin_dir_impl(root_dir.into(), x86_bin, x64_bin, arch)
}

fn try_bin_dir_impl(mut root_dir: PathBuf, x86_bin: &str, x64_bin: &str, arch: Arch) -> Option<PathBuf> {
    match arch {
        Arch::X86 => root_dir.push(x86_bin),
        Arch::X64 => root_dir.push(x64_bin),
    }

    if root_dir.is_dir() {
        Some(root_dir)
    } else {
        None
    }
}

fn try_rc_exe(mut pb: PathBuf) -> Option<PathBuf> {
    pb.push("rc.exe");
    if pb.exists() { Some(pb) } else { None }
}
