include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../build-support/swift.rs"
));

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        compile_swift_library("neosicht_wifi_native", &["src/adapters/core_wlan.swift"]);
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=CoreWLAN");
    }
}
