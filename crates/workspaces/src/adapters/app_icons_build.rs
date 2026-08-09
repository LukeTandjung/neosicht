fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/adapters/app_icons.m")
            .file("src/adapters/menus.m")
            .flag("-fobjc-arc")
            .compile("workspaces_native");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=ApplicationServices");
        println!("cargo:rerun-if-changed=src/adapters/app_icons.m");
        println!("cargo:rerun-if-changed=src/adapters/menus.m");
    }
}
