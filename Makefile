# Bequeme Targets rund um ./run.sh.
#
#   make up     baut und startet mongodb + backend + frontend
#   make help   zeigt alle Targets
#
# Wer kein make hat, nutzt direkt ./run.sh - das braucht nur Docker.

COMPOSE ?= docker compose

.DEFAULT_GOAL := help
.PHONY: help up start stop down logs ps build rebuild clean lockfiles mock

help: ## Zeigt diese Uebersicht
	@grep -hE '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "  Frontend: http://localhost:8686   Backend: http://localhost:8585"

up: ## Baut und startet alles im Vordergrund (Strg-C beendet)
	./run.sh

start: ## Wie 'up', laeuft aber im Hintergrund weiter
	./run.sh -d

stop: ## Stoppt die Container, Datenbank-Inhalt bleibt erhalten
	$(COMPOSE) stop

down: ## Stoppt und entfernt die Container
	$(COMPOSE) down

logs: ## Haengt sich an die Logs aller Services
	$(COMPOSE) logs -f

ps: ## Zeigt den Status der Services
	$(COMPOSE) ps

build: lockfiles ## Baut die Images ohne zu starten
	$(COMPOSE) build

rebuild: lockfiles ## Baut alles ohne Cache neu (wenn ein Build unerklaerlich haengt)
	$(COMPOSE) build --no-cache

clean: ## Entfernt Container UND Volumes - die Datenbank ist danach leer
	$(COMPOSE) down -v

lockfiles: ## Erzeugt fehlende Lockfiles (backend/go.sum, frontend/package-lock.json)
	./run.sh --bootstrap

mock: ## Startet den Python-Mock-API-Server (Alternative zum Go-Backend)
	cd mock_api_server && ./start.sh
