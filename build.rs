use std::env;

#[cfg(target_os = "windows")]
extern crate winres;

fn main() {
    // let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));

    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "windows")]
    if target.contains("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/tsh.ico");
        res.set("ProductName", "Binary Patcher");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileDescription", "A Rust application for patching binaries");
        res.compile().unwrap();

        if target.contains("i686") {
            println!("cargo:rustc-link-arg=/FORCE:MULTIPLE");
            println!("cargo:rustc-link-lib=shlwapi");
        }
    }
}