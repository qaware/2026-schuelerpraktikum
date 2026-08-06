{ pkgs ? import <nixpkgs> {} }:

# Dev-Shell fuer das gesamte Projekt: Backend (Go), Frontend (Rust/WASM),
# Mock-API und Tooling. Aufruf: `nix-shell` im Repo-Root.
#
# Fuer die reine Frontend-Arbeit gibt es zusaetzlich ./frontend/shell.nix.
pkgs.mkShell {
  buildInputs = with pkgs; [
    # Backend
    go

    # Frontend
    cargo
    rustc
    rustup
    trunk
    nodejs_22
    lld

    # Mock-API-Server / DataGenerator
    python3

    # Alles zusammen starten
    docker-compose
    gnumake
  ];

  shellHook = ''
    if ! rustup show active-toolchain > /dev/null 2>&1; then
      echo "Keine Rust-Toolchain gefunden. Installiere 'stable'..."
      rustup default stable
    fi

    rustup target add wasm32-unknown-unknown

    echo ""
    echo "Dev-Shell bereit. Naechster Schritt: 'make up'"
    echo "  make up     -> baut und startet mongodb + backend + frontend"
    echo "  make help   -> alle Targets"
    echo ""
  '';
}
