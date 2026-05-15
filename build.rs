//! Extracted from https://github.com/cargo-bins/cargo-binstall/blob/main/crates/bin/build.rs
use std::thread;

const RERUN_INSTRUCTIONS: &str = "cargo:rerun-if-changed=build.rs
cargo:rerun-if-changed=manifest.rc
cargo:rerun-if-changed=windows.manifest";

fn main() {
    thread::scope(|s| {
        let handle = s.spawn(|| {
            println!("{RERUN_INSTRUCTIONS}");

            embed_resource::compile("manifest.rc", embed_resource::NONE)
                .manifest_required()
                .unwrap();
        });

        handle.join().unwrap();
    });
}
