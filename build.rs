use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let rtad_root = manifest_dir.join("rtad");

    // Auto-clone rtad repository if it doesn't exist
    if !rtad_root.exists() {
        println!("cargo:warning=Cloning rtad repository...");
        match git2::Repository::clone("https://github.com/dangjinghao/rtad.git", &rtad_root) {
            Ok(_) => println!("cargo:warning=rtad cloned successfully"),
            Err(e) => panic!("Failed to clone rtad: {}", e),
        }
    }

    let rtad_src = rtad_root.join("src/rtad.c");
    let rtad_include = rtad_root.join("include");
    let rtad_src_dir = rtad_root.join("src");
    let header_path = rtad_include.join("rtad.h");

    // Compile rtad C library
    cc::Build::new()
        .file(&rtad_src)
        .include(&rtad_include)
        .include(&rtad_src_dir)
        .warnings(false)
        .compile("rtad");

    println!("cargo:rustc-link-lib=static=rtad");
    println!("cargo:rerun-if-changed={}", rtad_src.display());
    println!("cargo:rerun-if-changed={}", header_path.display());

    // Generate Rust bindings using bindgen
    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .clang_arg(format!("-I{}", rtad_include.display()))
        .allowlist_function("rtad_.*")
        .generate_comments(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write bindings");
}
