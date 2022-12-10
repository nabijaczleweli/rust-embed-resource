# rust-embed-resource [![TravisCI build status](https://travis-ci.org/nabijaczleweli/rust-embed-resource.svg?branch=master)](https://travis-ci.org/nabijaczleweli/rust-embed-resource) [![AppVeyorCI build status](https://ci.appveyor.com/api/projects/status/nqd8kaa2pgwyiqkk/branch/master?svg=true)](https://ci.appveyor.com/project/nabijaczleweli/rust-embed-resource/branch/master) [![Licence](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](LICENSE) [![Crates.io version](https://img.shields.io/crates/v/embed-resource)](https://crates.io/crates/embed-resource)
A [`Cargo` build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) library to handle compilation and inclusion of Windows resources
in the most resilient fashion imaginable

## [Documentation](https://rawcdn.githack.com/nabijaczleweli/rust-embed-resource/doc/embed_resource/index.html)

## Quickstart

In your build script, assuming the resource file is called `checksums.rc`:

```rust
extern crate embed_resource;

fn main() {
    // Compile and link checksums.rc
    embed_resource::compile("checksums.rc");

    // Or, to select a resource file for each binary separately
    embed_resource::compile_for("assets/poke-a-mango.rc", &["poke-a-mango", "poke-a-mango-installer"]);
    embed_resource::compile_for("assets/uninstaller.rc", &["unins001"]);
}
```

## Example: Embedding a Windows Manifest
Courtesy of [@jpoles1](https://github.com/jpoles1).

The following steps are used to embed a manifest in your compiled rust .exe file. In this example the manifest will cause admin permissions to be requested for the final executable:

1. Add the following to your cargo.toml:
```toml
[build-dependencies]
embed-resource = "1.8"
```

2. In your project root directory, add a file named `build.rs` with the following:
```rust
extern crate embed_resource;
fn main() {
    embed_resource::compile("app-name-manifest.rc");
}
```

3. In your project root directory, add a file named `app-name-manifest.rc` with the following:
```c
#define RT_MANIFEST 24
1 RT_MANIFEST "app-name.exe.manifest"
```

4. In your project root directory, add a file named `app-name.exe.manifest` with the following:
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
</assembly>
```

5. Build your project!

## Errata

If no `cargo:rerun-if-changed` annotations are generated, Cargo scans the entire build root by default.
Because the first step in building a manifest is an unspecified C preprocessor step with-out the ability to generate the equivalent of `cc -MD`, we do *not* output said annotation.

If scanning is prohibitively expensive, or you have something else that generates the annotations, you may want to spec the full non-system dependency list for your manifest manually, so:
```rust
println!("cargo:rerun-if-changed=app-name-manifest.rc");
embed_resource::compile("app-name-manifest.rc");
```
for the above example (cf. [#41](https://github.com/nabijaczleweli/rust-embed-resource/issues/41)).

## Credit

In chronological order:

[@liigo](https://github.com/liigo) -- persistency in pestering me and investigating problems where I have failed

[@mzji](https://github.com/mzji) -- MSVC lab rat

[@TheCatPlusPlus](https://github.com/TheCatPlusPlus) -- knowledge and providing first iteration of manifest-embedding code

[@azyobuzin](https://github.com/azyobuzin) -- providing code for finding places where RC.EXE could hide

[@retep998](https://github.com/retep998) -- fixing MSVC support

[@SonnyX](https://github.com/SonnyX) -- Windows cross-compilation support and testing

[@MSxDOS](https://github.com/MSxDOS) -- finding and supplying RC.EXE its esoteric header include paths

[@roblabla](https://github.com/roblabla) -- cross-compilation to Windows MSVC via LLVM-RC

## Special thanks

To all who support further development on Patreon, in particular:

  * ThePhD
  * Embark Studios
  * Lars Strojny
