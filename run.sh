#!/usr/bin/env sh
#
# Startet das komplette Projekt (mongodb + backend + frontend).
# Einzige Voraussetzung: Docker. Kein Go, kein npm, kein nix noetig.
#
#   ./run.sh                startet alles im Vordergrund
#   ./run.sh -d             startet im Hintergrund
#   ./run.sh --bootstrap    erzeugt nur die fehlenden Lockfiles
#
# Weitere Argumente werden an 'docker compose up' durchgereicht.

set -eu

cd "$(dirname "$0")"

GO_IMAGE=${GO_IMAGE:-golang:1.26-alpine}
NODE_IMAGE=${NODE_IMAGE:-node:22-bookworm-slim}

die() {
	echo "FEHLER: $1" >&2
	exit 1
}

command -v docker >/dev/null 2>&1 || die "Docker ist nicht installiert."
docker compose version >/dev/null 2>&1 || die "'docker compose' ist nicht verfuegbar (Compose-Plugin fehlt)."

# go.sum und package-lock.json liegen nicht im Repo, werden von den Dockerfiles
# aber gebraucht ('go mod download' bzw. 'npm ci'). Beide werden hier in
# Wegwerf-Containern erzeugt, wenn sie fehlen oder veraltet sind.
generate() {
	target=$1
	source=$2
	shift 2

	if [ -f "$target" ] && [ ! "$source" -nt "$target" ]; then
		return 0
	fi

	echo ">> erzeuge $target"
	# Erst in eine temporaere Datei, damit ein Abbruch keine leere,
	# scheinbar gueltige Datei zuruecklaesst.
	if ! "$@" > "$target.tmp" 2>/dev/null || [ ! -s "$target.tmp" ]; then
		rm -f "$target.tmp"
		die "$target konnte nicht erzeugt werden."
	fi
	mv "$target.tmp" "$target"
}

bootstrap() {
	# 'go mod tidy' schreibt ins Arbeitsverzeichnis, deshalb wird das Modul im
	# Container kopiert und das Ergebnis ueber stdout herausgelesen.
	generate backend/go.sum backend/go.mod \
		docker run --rm -v "$PWD/backend:/src:ro" "$GO_IMAGE" \
		sh -c 'cp -r /src /work && cd /work && go mod tidy >/dev/null 2>&1 && cat go.sum'

	generate frontend/package-lock.json frontend/package.json \
		docker run --rm -v "$PWD/frontend/package.json:/src/package.json:ro" "$NODE_IMAGE" \
		sh -c 'mkdir -p /work && cp /src/package.json /work/ && cd /work && npm install --package-lock-only --silent >/dev/null 2>&1 && cat package-lock.json'
}

if [ "${1:-}" = "--bootstrap" ]; then
	bootstrap
	echo "Lockfiles sind aktuell."
	exit 0
fi

bootstrap

echo ">> baue und starte Container"
docker compose up --build "$@"

# Bei -d kehrt compose sofort zurueck, dann sind die URLs hilfreich.
case " $* " in
*" -d "* | *" --detach "*)
	echo ""
	echo "Laeuft. Frontend: http://localhost:8686 | Backend: http://localhost:8585"
	echo "Logs: docker compose logs -f   Stoppen: docker compose down"
	;;
esac
