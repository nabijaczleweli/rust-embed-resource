//! A [`Cargo` build script](http://doc.crates.io/build-script.html) library to handle compilation and inclusion of Windows
//! resources in the most resilient fashion imaginable
//!
//! # Background
//!
//! Including Windows resources seems very easy at first, despite the build scripts' abhorrent documentation:
//! [compile with `windres`, then make linkable with `ar`]
//! (https://github.com/nabijaczleweli/cargo-update/commit/ef4346c#diff-a7b0a2dee0126cddf994326e705a91ea).
//!
//! I was very happy with that solution until it was brought to my attention, that [MSVC uses something different]
//! (https://github.com/nabijaczleweli/cargo-update/commit/f57e9c3#diff-a7b0a2dee0126cddf994326e705a91ea),
//! and now either `windres`-`ar` combo or `RC.EXE` would be used, which was OK.
//!
//! Later it transpired, that [MSVC is even more incompatible with everything else]
//! (https://github.com/nabijaczleweli/cargo-update/commit/39fa758#diff-a7b0a2dee0126cddf994326e705a91ea)
//! by way of not having `RC.EXE` in `$PATH` (because it would only be reasonable to do so),
//! so another MSVC artisan made the script [find the most likely places for `RC.EXE` to be]
//! (https://github.com/nabijaczleweli/cargo-update/pull/22), and the script grew yet again,
//! now standing at 100 lines and 3.2 kB.
//!
//! After [copying the build script in its entirety]
//! (https://github.com/thecoshman/http/commit/98205a4#diff-a7b0a2dee0126cddf994326e705a91ea)
//! and realising how error-prone that was, then being [nudged by Shepmaster]
//! (https://chat.stackoverflow.com/transcript/message/35378953#35378953)
//! to extract it to a crate, here we are.
//!
//! # Usage (overview)
//!
//! Since the [build script](http://doc.crates.io/build-script.html) documentation is trash and
//! [build script handling is even more trash]
//! (https://github.com/nabijaczleweli/cargo-update/commit/ef4346c#diff-639fbc4ef05b315af92b4d836c31b023),
//! we can't print a build script line to link to the compiled resource file. Instead, you need to use and `extern`
//! block in your `main.rs` file like so:
//!
//! ```rust,ignore
//! #[cfg(target_os="windows")]
//! #[link(name="checksums-manifest", kind="static")]
//! extern "C" {}
//!
//! // Your main() and w/e
//! ```
//!
//! Since the manifest is only generated on Windows, the `cfg` attribute takes care of that.
//!
//! The `name` attribute argument is either:
//!
//!   * The name of the crate + "-manifest" by default, or
//!   * The second argument to `compile()`.
//!
//! # Usage (detailed)
//!
//! For the purposes of the demonstration we will assume that the crate's name is "checksums" and that the resource file's name
//! is "checksums.rc".
//!
//! In `Cargo.toml`:
//!
//! ```toml
//! # The general section with crate name, license, etc.
//! build = "build.rs"
//!
//! [build-dependencies]
//! embed-resource = "1.0"
//! ```
//!
//! In `build.rs`:
//!
//! ```rust,no-run
//! extern crate embed_resource;
//!
//! fn main() {
//!     embed_resource::compile("checksums.rc", None, None);
//! }
//! ```
//!
//! In `main.rs`:
//!
//! ```rust,ignore
//! #[cfg(target_os="windows")]
//! #[link(name="checksums-manifest", kind="static")]
//! extern "C" {}
//! ```
//!
//! If, however, you want to use a different manifest link name (here: "chksum-rc"):
//!
//! In `build.rs`:
//!
//! ```rust,no-run
//! extern crate embed_resource;
//!
//! fn main() {
//!     embed_resource::compile("checksums.rc", Some("chksum-rc"), None);
//! }
//! ```
//!
//! In `main.rs`:
//!
//! ```rust,ignore
//! #[cfg(target_os="windows")]
//! #[link(name="chksum-rc", kind="static")]
//! extern "C" {}
//! ```
//!
//! If, for an unfathomable reason, you want to create the resource archive in a different location:
//!
//! In `build.rs`:
//!
//! ```rust,no-run
//! extern crate embed_resource;
//!
//! fn main() {
//!     embed_resource::compile("checksums.rc", None, Some("C:/Rast/build-files-output"));
//! }
//! ```
//!
//! I souldn't recommend this, but hey.
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


/// Compile the Windows resource file and update the cargo search path if we're on Windows.
///
/// `prefix`, if `None`, defaults to `"$CARGO_PKG_NAME-manifest"`, this is the name you'll link to in `main.rs`.
///
/// `out_dir`, if `None`, defaults to `$OUT_DIR`, which you probably don't want to change.
///
/// On non-Windows this does nothing, on non-MSVC Windows, this chains `windres` with `ar`,
/// but on MSVC Windows, this will try its hardest to find `RC.EXE` in Windows Kits and/or SDK directories
/// (because someone thought not putting it in `$PATH` was a great idea).
///
/// # Examples
///
/// In your build script, assuming the crate's name is "checksums":
///
/// ```rust,no-run
/// extern crate embed_resource;
///
/// fn main() {
///     // Compile file checksums.rc to be linkable as checksums-manifest in $OUT_DIR
///     embed_resource::compile("checksums.rc", None, None);
/// }
/// ```
///
/// If you want to link as `chksum-rc`:
///
/// ```rust,no-run
/// extern crate embed_resource;
///
/// fn main() {
///     // Compile file checksums.rc to be linkable as chksum-rc in $OUT_DIR
///     embed_resource::compile("checksums.rc", Some("chksum-rc"), None);
/// }
/// ```
pub fn compile(resource_file: &str, prefix: Option<&str>, out_dir: Option<&str>) {
    if SUPPORTED {
        let prefix = prefix.map_or_else(|| env::var("CARGO_PKG_NAME").unwrap() + "-manifest", str::to_string);
        let out_dir = out_dir.map_or_else(|| env::var("OUT_DIR").unwrap(), str::to_string);

        compile_resource(&out_dir, &prefix, resource_file);
        println!("cargo:rustc-link-search=native={}", out_dir);
    }
}
