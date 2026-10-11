{ lib, rustPlatform, makeWrapper, dunst }:
rustPlatform.buildRustPackage {
  pname = "waybar-dnd";
  version = "0.1.0";
  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [ ./Cargo.toml ./Cargo.lock ./src ./tests/cli.rs ./examples ];
  };
  cargoLock.lockFile = ./Cargo.lock;
  nativeBuildInputs = [ makeWrapper ];
  strictDeps = true;
  doCheck = true;

  postInstall = ''
    wrapProgram "$out/bin/waybar-dnd" \
      --prefix PATH : ${lib.makeBinPath [ dunst ]}
    install -Dm644 examples/waybar.jsonc "$out/share/waybar-dnd/waybar.jsonc"
  '';

  meta = {
    description = "dnd switch for waybar";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "waybar-dnd";
  };
}
