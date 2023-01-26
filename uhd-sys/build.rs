extern crate bindgen;
extern crate metadeps;

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=uhd");
    println!("cargo:rustc-link-lib=boost_chrono");
    println!("cargo:rustc-link-lib=boost_date_time");
    println!("cargo:rustc-link-lib=boost_filesystem");
    println!("cargo:rustc-link-lib=boost_regex");
    println!("cargo:rustc-link-lib=boost_system");
    println!("cargo:rustc-link-lib=boost_thread");
    println!("cargo:rustc-link-lib=c");
    println!("cargo:rustc-link-lib=erio");
    // println!("cargo:rustc-link-lib=gcc_s"); // Doesn't seem to be needed, but I had this for some reason earlier...
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=udev");

    dotenv::dotenv().ok();
    let uhd_include_path = if let Ok(uhd_path) = env::var("UHD_DIR") {
        // Use the path provided via environmental variable

        let uhd_path = Path::new(&uhd_path);
        let this_crate_path = env::var("CARGO_MANIFEST_DIR").unwrap();
        println!(
            r"cargo:rustc-link-search={}",
            Path::new(&this_crate_path)
                .join(uhd_path)
                .join("lib")
                .to_str()
                .unwrap()
        );
        uhd_path.to_owned().join("include")
    } else {
        // This reads the metadata in Cargo.toml and sends Cargo the appropriate output to link the
        // libraries
        let libraries = metadeps::probe().unwrap();
        libraries
            .get("uhd")
            .expect("uhd library not in map")
            .include_paths
            .get(0)
            .expect("no include path for UHD headers")
            .to_owned()
    };
    generate_bindings(&uhd_include_path);
}

fn generate_bindings(include_path: &Path) {
    let usrp_header = include_path.join("uhd.h");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_path = out_dir.join("bindgen.rs");

    let mut builder = bindgen::builder()
        // .clang_arg("--target=armv7_unknown_linux_musleabihf")
        // .clang_arg("-IC:\\Users\\groundstation\\projects\\assets\\include")
        .whitelist_function("^uhd.+")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .header(usrp_header.to_string_lossy().to_owned())
        // Add the include directory to ensure that #includes in the header work correctly
        .clang_arg(format!("-I{}", include_path.to_string_lossy().to_owned()));

    // On Raspberry Pi devices, the include directories require some adjustment.
    let target = env::var("TARGET").expect("No TARGET environment variable");
    if target == "armv7-unknown-linux-gnueabihf" {
        builder = builder.clang_arg("-I/usr/lib/gcc/arm-linux-gnueabihf/8/include");
    }

    let bindings = builder.generate().expect("Failed to generate bindings");
    bindings
        .write_to_file(out_path)
        .expect("Failed to write bindings to file");
}
