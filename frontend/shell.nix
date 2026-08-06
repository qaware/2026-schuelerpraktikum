{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    rustc
    rustup
    trunk
    nodejs_22
    lld
  ];

  shellHook = ''
    if ! rustup show active-toolchain > /dev/null 2>&1; then
      echo "Keine Rust-Toolchain gefunden. Installiere 'stable'..."
      rustup default stable
    fi
    
    rustup target add wasm32-unknown-unknown
  '';
}