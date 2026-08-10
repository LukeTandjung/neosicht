use std::path::PathBuf;
use std::process::Command;

pub fn compile_swift_library(name: &str, sources: &[&str]) {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let library = out_dir.join(format!("lib{name}.a"));
    let manifest = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("manifest directory"),
    );
    let developer = PathBuf::from("/Applications/Xcode.app/Contents/Developer");
    let compiler = developer.join("Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc");
    let sdk = developer.join("Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk");
    let mut command = Command::new(compiler);
    command
        .env_remove("SDKROOT")
        .env_remove("DEVELOPER_DIR")
        .arg("-sdk")
        .arg(sdk)
        .arg("-target")
        .arg(format!("{}-apple-macosx14.0", std::env::var("CARGO_CFG_TARGET_ARCH").expect("target architecture")))
        .arg("-parse-as-library")
        .arg("-emit-library")
        .arg("-static")
        .arg("-module-name")
        .arg(name)
        .arg("-o")
        .arg(&library);
    for source in sources {
        command.arg(manifest.join(source));
        println!("cargo:rerun-if-changed={source}");
    }
    let output = command.output().expect("swiftc must be available");
    if !output.status.success() {
        panic!(
            "swiftc failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={name}");
    println!(
        "cargo:rustc-link-search=native=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/lib/swift"
    );
    println!(
        "cargo:rustc-link-search=native=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx"
    );
    println!("cargo:rustc-link-search=native=/usr/lib/swift");
    println!("cargo:rustc-link-lib=dylib=swiftCore");
}
