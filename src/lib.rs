//! A [`Cargo` build script](http://doc.crates.io/build-script.html) library to handle compilation and inclusion of Windows
//! resources in the most resilient fashion imaginable
//!
//! # Background
//!
//! Including Windows resources seems very easy at first, despite the build scripts' abhorrent documentation:
//! [compile with `windres`, then make linkable with
//! `ar`](https://github.com/nabijaczleweli/cargo-update/commit/ef4346c#diff-a7b0a2dee0126cddf994326e705a91ea).
//!
//! I was very happy with that solution until it was brought to my attention, that [MSVC uses something
//! different](https://github.com/nabijaczleweli/cargo-update/commit/f57e9c3#diff-a7b0a2dee0126cddf994326e705a91ea),
//! and now either `windres`-`ar` combo or `RC.EXE` would be used, which was OK.
//!
//! Later it transpired, that [MSVC is even more incompatible with everything
//! else](https://github.com/nabijaczleweli/cargo-update/commit/39fa758#diff-a7b0a2dee0126cddf994326e705a91ea)
//! by way of not having `RC.EXE` in `$PATH` (because it would only be reasonable to do so),
//! so another MSVC artisan made the script [find the most likely places for `RC.EXE` to
//! be](https://github.com/nabijaczleweli/cargo-update/pull/22), and the script grew yet again,
//! now standing at 100 lines and 3.2 kB.
//!
//! After [copying the build script in its
//! entirety](https://github.com/thecoshman/http/commit/98205a4#diff-a7b0a2dee0126cddf994326e705a91ea)
//! and realising how error-prone that was, then being [nudged by
//! Shepmaster](https://chat.stackoverflow.com/transcript/message/35378953#35378953)
//! to extract it to a crate, here we are.
//!
//! # Usage
//!
//! For the purposes of the demonstration we will assume that the resource file's name
//! is "checksums.rc", but it can be any name relative to the crate root.
//!
//! In `Cargo.toml`:
//!
//! ```toml
//! # The general section with crate name, license, etc.
//! build = "build.rs"
//!
//! [build-dependencies]
//! embed-resource = "1.8"
//! ```
//!
//! In `build.rs`:
//!
//! ```rust,no_run
//! extern crate embed_resource;
//!
//! fn main() {
//!     embed_resource::compile("checksums.rc");
//! }
//! ```
//!
//! ## Errata
//!
//! If no `cargo:rerun-if-changed` annotations are generated, Cargo scans the entire build root by default.
//! Because the first step in building a manifest is an unspecified C preprocessor step with-out the ability to generate the equivalent of `cc -MD`, we do *not* output said annotation.
//!
//! If scanning is prohibitively expensive, or you have something else that generates the annotations, you may want to spec the full non-system dependency list for your manifest manually, so:
//! ```rust
//! println!("cargo:rerun-if-changed=app-name-manifest.rc");
//! embed_resource::compile("app-name-manifest.rc");
//! ```
//! for the above example (cf. [#41](https://github.com/nabijaczleweli/rust-embed-resource/issues/41)).
//!
//! # Cross-compilation
//!
//! It is possible to embed resources in Windows executables built on non-Windows hosts. There are two ways to do this:
//!
//! When targetting `*-pc-windows-gnu`, `*-w64-mingw32-windres` is attempted by default, for `*-pc-windows-msvc` it's `llvm-rc`,
//! this can be overriden by setting `RC_$TARGET`, `RC_${TARGET//-/_}`, or `RC` environment variables.
//!
//! When compiling with LLVM-RC, an external C compiler is used to preprocess the resource,
//! preloaded with configuration from
//! [`cc`](https://github.com/alexcrichton/cc-rs#external-configuration-via-environment-variables).
//!
//! # Credit
//!
//! In chronological order:
//!
//! [@liigo](https://github.com/liigo) -- persistency in pestering me and investigating problems where I have failed
//!
//! [@mzji](https://github.com/mzji) -- MSVC lab rat
//!
//! [@TheCatPlusPlus](https://github.com/TheCatPlusPlus) -- knowledge and providing first iteration of manifest-embedding code
//!
//! [@azyobuzin](https://github.com/azyobuzin) -- providing code for finding places where RC.EXE could hide
//!
//! [@retep998](https://github.com/retep998) -- fixing MSVC support
//!
//! [@SonnyX](https://github.com/SonnyX) -- Windows cross-compilation support and testing
//!
//! [@MSxDOS](https://github.com/MSxDOS) -- finding and supplying RC.EXE its esoteric header include paths
//!
//! [@roblabla](https://github.com/roblabla) -- cross-compilation to Windows MSVC via LLVM-RC
//!
//! # Special thanks
//!
//! To all who support further development on [Patreon](https://patreon.com/nabijaczleweli), in particular:
//!
//!   * ThePhD
//!   * Embark Studios
//!   * Lars Strojny


#[cfg(any(not(target_os = "windows"), all(target_os = "windows", target_env = "msvc")))]
extern crate cc;
extern crate toml;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
extern crate vswhom;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
extern crate winreg;
extern crate rustc_version;

#[cfg(not(target_os = "windows"))]
mod non_windows;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
mod windows_msvc;
#[cfg(all(target_os = "windows", not(target_env = "msvc")))]
mod windows_not_msvc;

#[cfg(not(target_os = "windows"))]
use self::non_windows::*;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
use self::windows_msvc::*;
#[cfg(all(target_os = "windows", not(target_env = "msvc")))]
use self::windows_not_msvc::*;

use std::{env, fs};
use std::fmt::Display;
use toml::Value as TomlValue;
use toml::map::Map as TomlMap;
use std::path::{Path, PathBuf};


/// Compile the Windows resource file and update the cargo search path if building for Windows.
///
/// On non-Windows non-Windows-cross-compile-target this does nothing, on non-MSVC Windows and Windows cross-compile targets,
/// this chains `windres` with `ar`,
/// but on MSVC Windows, this will try its hardest to find `RC.EXE` in Windows Kits and/or SDK directories,
/// falling back to [Jon Blow's VS discovery script](https://pastebin.com/3YvWQa5c),
/// and on Windows 10 `%INCLUDE%` will be updated to help `RC.EXE` find `windows.h` and friends.
///
/// `$OUT_DIR` is added to the include search path.
///
/// Note that this does *nothing* if building with rustc before 1.50.0 and there's a library in the crate,
/// since the resource is linked to the library, if any, instead of the binaries.
///
/// Since rustc 1.50.0, the resource is linked only to the binaries
/// (unless there are none, in which case it's also linked to the library).
///
/// # Examples
///
/// In your build script, assuming the crate's name is "checksums":
///
/// ```rust,no_run
/// extern crate embed_resource;
///
/// fn main() {
///     // Compile and link checksums.rc
///     embed_resource::compile("checksums.rc");
/// }
/// ```
pub fn compile<T: AsRef<Path>>(resource_file: T) {
    if let Some((prefix, out_dir, out_file)) = compile_impl(resource_file.as_ref()) {
        let hasbins = fs::read_to_string("Cargo.toml")
            .unwrap_or_else(|err| {
                eprintln!("Couldn't read Cargo.toml: {}; assuming src/main.rs or S_ISDIR(src/bin/)", err);
                String::new()
            })
            .parse::<TomlValue>()
            .unwrap_or_else(|err| {
                eprintln!("Couldn't parse Cargo.toml: {}; assuming src/main.rs or S_ISDIR(src/bin/)", err);
                TomlValue::Table(TomlMap::new())
            })
            .as_table()
            .map(|t| t.contains_key("bin"))
            .unwrap_or(false) || (Path::new("src/main.rs").exists() || Path::new("src/bin").is_dir());
        eprintln!("Final verdict: crate has binaries: {}", hasbins);

        if hasbins && rustc_version::version().expect("couldn't get rustc version") >= rustc_version::Version::new(1, 50, 0) {
            println!("cargo:rustc-link-arg-bins={}", out_file);
        } else {
            // Cargo pre-0.51.0 (rustc pre-1.50.0) compat
            // Only links to the calling crate's library
            println!("cargo:rustc-link-search=native={}", out_dir);
            println!("cargo:rustc-link-lib=dylib={}", prefix);
        }
    }
}

/// Likewise, but only for select binaries.
///
/// Only available since rustc 1.55.0, does nothing before.
///
/// # Examples
///
/// ```rust,no_run
/// extern crate embed_resource;
///
/// fn main() {
///     embed_resource::compile_for("assets/poke-a-mango.rc", &["poke-a-mango", "poke-a-mango-installer"]);
///     embed_resource::compile_for("assets/uninstaller.rc", &["unins001"]);
/// }
/// ```
pub fn compile_for<T: AsRef<Path>, J: Display, I: IntoIterator<Item = J>>(resource_file: T, for_bins: I) {
    if let Some((_, _, out_file)) = compile_impl(resource_file.as_ref()) {
        for bin in for_bins {
            println!("cargo:rustc-link-arg-bin={}={}", bin, out_file);
        }
    }
}

fn compile_impl(resource_file: &Path) -> Option<(&str, String, String)> {
    let comp = ResourceCompiler::new();
    if comp.is_supported() {
        let prefix = &resource_file.file_stem().expect("resource_file has no stem").to_str().expect("resource_file's stem not UTF-8");
        let out_dir = env::var("OUT_DIR").expect("No OUT_DIR env var");

        let out_file = comp.compile_resource(&out_dir, &prefix, resource_file.to_str().expect("resource_file not UTF-8"));
        Some((prefix, out_dir, out_file))
    } else {
        None
    }
}


/// Find MSVC build tools other than the compiler and linker
///
/// On Windows + MSVC this can be used try to find tools such as `MIDL.EXE` in Windows Kits and/or SDK directories.
///
/// The compilers and linkers can be better found with the `cc` or `vswhom` crates.
/// This always returns `None` on non-MSVC targets.
///
/// # Examples
///
/// In your build script, find `midl.exe` and use it to compile an IDL file:
///
/// ```rust,no_run
/// # #[cfg(all(target_os = "windows", target_env = "msvc"))]
/// # {
/// extern crate embed_resource;
/// extern crate vswhom;
/// # use std::env;
/// # use std::process::Command;
///
/// let midl = embed_resource::find_windows_sdk_tool("midl.exe").unwrap();
///
/// // midl.exe uses cl.exe as a preprocessor, so it needs to be in PATH
/// let vs_locations = vswhom::VsFindResult::search().unwrap();
/// let output = Command::new(midl)
///     .env("PATH", vs_locations.vs_exe_path.unwrap())
///     .args(&["/out", &env::var("OUT_DIR").unwrap()])
///     .arg("haka.pfx.idl").output().unwrap();
///
/// assert!(output.status.success());
/// # }
/// ```
pub fn find_windows_sdk_tool<T: AsRef<str>>(tool: T) -> Option<PathBuf> {
    find_windows_sdk_tool_impl(tool.as_ref())
}
