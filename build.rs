use std::env;

#[cfg(target_os = "windows")]
extern crate winres;

fn main() {
    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(target_os = "windows")]
    if target.contains("windows") && env::var("PROFILE").unwrap() == "release" {
        println!("cargo:rerun-if-changed=res/tsh.ico");
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/tsh.ico")
            .set_language(0x0409);
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.compile().unwrap();
    }
    if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=ncurses");
        return;
    }
    if target.contains("unknown") {
        return;
    }
}
