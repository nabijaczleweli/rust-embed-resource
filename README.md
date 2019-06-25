# rust-embed-resource [![TravisCI build status](https://travis-ci.org/nabijaczleweli/rust-embed-resource.svg?branch=master)](https://travis-ci.org/nabijaczleweli/rust-embed-resource) [![AppVeyorCI build status](https://ci.appveyor.com/api/projects/status/nqd8kaa2pgwyiqkk/branch/master?svg=true)](https://ci.appveyor.com/project/nabijaczleweli/rust-embed-resource/branch/master) [![Licence](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](LICENSE) [![Crates.io version](https://meritbadge.herokuapp.com/embed-resource)](https://crates.io/crates/embed-resource)
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
}
```

## Credit

In chronological order:

[@liigo](https://github.com/liigo) -- persistency in pestering me and investigating problems where I have failed

[@mzji](https://github.com/mzji) -- MSVC lab rat

[@TheCatPlusPlus](https://github.com/TheCatPlusPlus) -- knowledge and providing first iteration of manifest-embedding code

[@azyobuzin](https://github.com/azyobuzin) -- providing code for finding places where RC.EXE could hide

[@retep998](https://github.com/retep998) -- fixing MSVC support

## Special thanks

To all who support further development on Patreon, in particular:

  * ThePhD
