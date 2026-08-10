include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../build-support/swift.rs"
));

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        compile_swift_library("neosicht_datetime_native", &["src/adapters/eventkit.swift"]);
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=EventKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}
