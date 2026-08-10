use std::path::PathBuf;
use std::process::Command;

pub fn compile_swift_library(name: &str, sources: &[&str]) {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let library = out_dir.join(format!("lib{name}.a"));
    let manifest = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("manifest directory"),
    );
    let developer = PathBuf::from("/Applications/Xcode.app/Contents/Developer");
    let configured_compiler = std::env::var_os("SWIFTC");
    let compiler = configured_compiler.clone().map(PathBuf::from).unwrap_or_else(|| {
        developer.join("Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc")
    });
    let sdk = configured_compiler
        .as_ref()
        .and_then(|_| std::env::var_os("SDKROOT"))
        .map(PathBuf::from)
        .unwrap_or_else(|| developer.join("Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"));
    let mut command = Command::new(compiler);
    if configured_compiler.is_none() {
        command.env_remove("DEVELOPER_DIR");
    }
    command
        .env("SDKROOT", &sdk)
        .arg("-sdk")
        .arg(&sdk)
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
    let default_runtime_paths = format!(
        "{}/usr/lib/swift:/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx:/usr/lib/swift",
        sdk.display()
    );
    let runtime_paths = std::env::var("SWIFT_RUNTIME_LIBRARY_PATHS")
        .unwrap_or(default_runtime_paths);
    for path in runtime_paths.split(':').filter(|path| !path.is_empty()) {
        println!("cargo:rustc-link-search=native={path}");
    }
    println!("cargo:rustc-link-lib=dylib=swiftCore");
}
