include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../build-support/swift.rs"
));

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("src/Info.plist");
        println!(
            "cargo:rustc-link-arg-bin=neosicht=-Wl,-sectcreate,__TEXT,__info_plist,{}",
            manifest.display()
        );
        println!("cargo:rerun-if-changed=src/Info.plist");

        compile_swift_library("neosicht_native", &["src/adapters/panel.swift"]);
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
    }
}
