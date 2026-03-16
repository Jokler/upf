{
  pkgs,
  lib,
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  pname = "upf";
  version = "0.1.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;

  nativeBuildInputs = [pkgs.pkg-config];
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

  meta = {
    description = "Uploads files based on templates";
    mainProgram = "upf";
  };
}
