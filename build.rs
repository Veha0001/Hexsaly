#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    use std::env;
    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));
    println!("cargo:rerun-if-changed=build.rs");
    if target.contains("windows") && env::var("PROFILE").unwrap() == "release" {
        println!("cargo:rerun-if-changed=res/tsh.ico");
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/tsh.ico").set_language(0x0409);
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.compile().unwrap();
    }
}
#[cfg(unix)]
fn main() {}
