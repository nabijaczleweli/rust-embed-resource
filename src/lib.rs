#[cfg(all(windows, target_env = "msvc"))]
extern crate winreg;

#[cfg(not(windows))]
mod non_windows;
#[cfg(all(windows, target_env = "msvc"))]
mod windows_msvc;
#[cfg(all(windows, not(target_env = "msvc")))]
mod windows_not_msvc;

#[cfg(not(windows))]
use self::non_windows::*;
#[cfg(all(windows, target_env = "msvc"))]
use self::windows_msvc::*;
#[cfg(all(windows, not(target_env = "msvc")))]
use self::windows_not_msvc::*;

use std::env;


pub fn compile(resource_file: &str, prefix: Option<&str>, out_dir: Option<&str>) {
    if SUPPORTED {
        let prefix = prefix.map_or_else(|| env::var("CARGO_PKG_NAME").unwrap() + "-manifest", str::to_string);
        let out_dir = out_dir.map_or_else(|| env::var("OUT_DIR").unwrap(), str::to_string);

        compile_resource(&out_dir, &prefix, resource_file);
        println!("cargo:rustc-link-search=native={}", out_dir);
    }
}
