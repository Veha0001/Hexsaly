use std::env;

#[cfg(target_os = "windows")]
extern crate winres;

fn main() {
    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));

    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "windows")]
    if target.contains("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/tsh.ico");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.compile().unwrap();
    }
}
