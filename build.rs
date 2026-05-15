extern crate cc;
fn main() {
    let mut cc = cc::Build::new();
    cc.cpp(true);
    cc.file("lib.cpp");
    cc.compile("library");
}
