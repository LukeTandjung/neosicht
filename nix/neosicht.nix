{
  lib,
  rustPlatform,
  swift,
  apple-sdk,
  jq,
}:
rustPlatform.buildRustPackage {
  pname = "neosicht";
  version = "0.1.0";

  src = lib.cleanSource ../.;
  cargoHash = "sha256-OBPMlQzCONrEVg0d5pODR3WAqReIdH+S9xSfT4G9JKI=";

  nativeBuildInputs = [swift jq];

  preBuild = ''
    export SWIFTC="$(command -v swiftc)"
    export SDKROOT="${apple-sdk}/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
    export SWIFT_RUNTIME_LIBRARY_PATHS="$($SWIFTC -print-target-info | ${lib.getExe jq} -r '.paths.runtimeLibraryPaths | join(":")'):$SDKROOT/usr/lib/swift"
  '';

  postInstall = ''
    app="$out/Applications/Neosicht.app/Contents"
    mkdir -p "$app/MacOS" "$app/Resources"
    mv "$out/bin/neosicht" "$app/MacOS/neosicht"
    cp crates/neosicht/src/Info.plist "$app/Info.plist"
    ln -s ../Applications/Neosicht.app/Contents/MacOS/neosicht "$out/bin/neosicht"
  '';

  meta = {
    description = "A GPUI desktop environment bar for macOS";
    homepage = "https://github.com/LukeTandjung/neosicht";
    license = lib.licenses.mit;
    mainProgram = "neosicht";
    platforms = lib.platforms.darwin;
  };
}
