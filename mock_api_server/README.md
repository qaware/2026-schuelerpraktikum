# Satellite Mock REST API Server

## Features der Daten-Generierung
- **Unregelmäßige Sendeintervalle:** Sensortypen senden in unterschiedlichen Intervallen (z. B. Steuerdüsen alle 1–3s, Gastanks alle 4–9s).
- **Rauschen & Ungenauigkeiten:** Druck & Temperatur enthalten verteiltes Rauschen (`Gaussian Noise`), variierende Nachkommastellen sowie vereinzelte Peaks & Spikes (z. B. kurze Druckanstiege).
- **Sensoraussetzer / Paketverluste:** Ca. 10–12 % der Übertragungszyklen fallen zufällig aus (Aussetzer einzelner Sensoren).
- **Konsistente Historie:** Beim Serverstart werden 60 Minuten historische Daten mit ungleichmäßigen Zeitabständen vorab erzeugt und dauerhaft gespeichert.

## Server starten

Wechsle in den Ordner und starte das Skript:

```bash
cd mock_api_server
./start.sh
```

*(Das Skript erstellt automatisch eine eigene Python Virtual environment `.venv` und installiert FastAPI & Uvicorn).*

Standardmäßig läuft der Server auf **Port 8000**:
- **Base URL:** `http://localhost:8000`
- **Interaktive Swagger-Dokumentation:** `http://localhost:8000/docs`

### Port anpassen

Falls Port 8000 belegt ist oder ein anderer Port gewünscht ist:
```bash
PORT=8080 ./start.sh
```

---

## Unterstützte Endpunkte (gemäß `api_spezifikation.md`)

| Methode | Pfad | Beschreibung |
| :--- | :--- | :--- |
| `GET` | `/satellites` | Liste aller verfügbaren Satelliten-Namen |
| `GET` | `/satellites/log` | Telemetrie-Logdaten **aller** Satelliten (`?amount=10` oder `/10`) |
| `GET` | `/satellites/{name}` | Details zu einem bestimmten Satelliten |
| `GET` | `/satellites/{name}/log` | Telemetrie-Logdaten für einen einzelnen Satelliten (`?amount=10` oder `/10`) |
| `GET` | `/satellites/{name}/sensors` | Liste aller bekannten Sensornamen |
| `GET` | `/satellites/{name}/sensors/{sensor_name}` | Telemetrie-Logdaten für einen einzelnen Sensor (`?amount=10` oder `/10`) |
| `GET` | `/error` | (Optional) Liste von Fehler-Timestamps |

### Parameter `amount`

Der Parameter `amount` (Standard: 100) kann sowohl als **Query-Parameter** als auch als **Pfad-Segment** übergeben werden:

**Query-Parameter (Empfohlen):**
```http
GET http://localhost:8000/satellites/log?amount=10
GET http://localhost:8000/satellites/ISS/log?amount=50
GET http://localhost:8000/satellites/ISS/sensors/thruster_1.a?amount=20
```

**Pfad-Parameter:**
```http
GET http://localhost:8000/satellites/log/10
GET http://localhost:8000/satellites/ISS/log/50
GET http://localhost:8000/satellites/ISS/sensors/thruster_1.a/20
```

---

## Beispiel-Antworten (JSON)

### 1. `GET /satellites`
```json
{
  "names": ["ISS", "Hubble", "JWST"]
}
```

### 2. `GET /satellites/ISS`
```json
{
  "name": "ISS",
  "model": "Zarya-1",
  "launchdate": "1998-11-20T06:40:00Z",
  "sensors": [
    "thruster_1.a", "thruster_1.b", "thruster_1.c",
    "oxygen_tank_1", "hydrogen_tank_1"
  ],
  "nation": "International"
}
```

### 3. `GET /satellites/ISS/sensors/thruster_1.a`
```json
{
  "amount": 1,
  "data": [
    {
      "sensor_name": "thruster_1.a",
      "pressure": 4.521834,
      "temperature": 352.149201,
      "position": {
        "city": "Miami",
        "height": 408.152341
      },
      "specs": {
        "name": "ISS",
        "model": "Zarya-1",
        "launch_date": "20.11.1998",
        "sensors": "thruster",
        "nation": "International"
      },
      "timestamp": 1785920473
    }
  ]
}
```
