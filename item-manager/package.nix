{
  lib,
  makeRustPlatform,
  rust-bin,
}:
let
  toolchain = rust-bin.stable.latest.default;
  rustPlatform = makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
rustPlatform.buildRustPackage {
  pname = "item-manager";
  version = "0.1.0";

  src =
    let
      fs = lib.fileset;
    in
    fs.toSource {
      root = ./.;
      fileset = fs.difference ./. (
        fs.unions [
          (fs.maybeMissing ./result)
        ]
      );
    };

  cargoDeps = rustPlatform.importCargoLock { lockFile = ./Cargo.lock; };

  doCheck = false;

  SQLX_OFFLINE = true;

  meta = {
    mainProgram = "app";
  };
}
