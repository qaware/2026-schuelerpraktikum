import asyncio
import datetime
import math
import random
from typing import Dict, List, Optional
from fastapi import FastAPI, Query, HTTPException
from fastapi.middleware.cors import CORSMiddleware
import uvicorn

app = FastAPI(
    title="Satellite Realtime Mock API",
    description="Realtime API Server mit unregelmäßigen Intervallen, Rauschen & Aussetzern",
    version="3.0.0",
)

# Enable CORS so frontends running on any port can access the API
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Satellite Data Catalog
SATELLITES_DB = {
    "ISS": {
        "name": "ISS",
        "model": "Zarya-1",
        "launch_date": "20.11.1998",
        "launchdate": "1998-11-20T06:40:00Z",
        "base_height": 408.15,
        "sensors": [
            "thruster_1.a", "thruster_1.b", "thruster_1.c",
            "thruster_2.a", "thruster_2.b", "thruster_2.c",
            "thruster_3.a", "thruster_3.b", "thruster_3.c",
            "oxygen_tank_1", "oxygen_tank_2",
            "hydrogen_tank_1", "hydrogen_tank_2"
        ],
        "nation": "International"
    },
    "Hubble": {
        "name": "Hubble",
        "model": "HST-1",
        "launch_date": "24.04.1990",
        "launchdate": "1990-04-24T12:33:51Z",
        "base_height": 540.22,
        "sensors": [
            "thruster_1.a", "thruster_1.b", "thruster_1.c",
            "oxygen_tank_1", "hydrogen_tank_1"
        ],
        "nation": "USA"
    },
    "JWST": {
        "name": "JWST",
        "model": "Webb-O1",
        "launch_date": "25.12.2021",
        "launchdate": "2021-12-25T12:20:00Z",
        "base_height": 1500000.0,
        "sensors": [
            "thruster_1.a", "thruster_1.b", "thruster_2.a",
            "oxygen_tank_1", "hydrogen_tank_1"
        ],
        "nation": "USA/ESA/CSA"
    }
}

# Baseline signal specifications per sensor with custom intervals and noise
SENSOR_SPECS = {
    "thruster_1.a": {"type": "thruster", "base_p": 4.5, "amp_p": 1.2, "base_t": 350.0, "amp_t": 25.0, "freq": 0.05, "interval": (1.5, 3.5), "p_noise": 0.15, "t_noise": 1.2},
    "thruster_1.b": {"type": "thruster", "base_p": 4.2, "amp_p": 1.0, "base_t": 340.0, "amp_t": 20.0, "freq": 0.04, "interval": (2.0, 4.0), "p_noise": 0.18, "t_noise": 1.5},
    "thruster_1.c": {"type": "thruster", "base_p": 4.8, "amp_p": 1.5, "base_t": 360.0, "amp_t": 30.0, "freq": 0.06, "interval": (1.0, 3.0), "p_noise": 0.20, "t_noise": 2.0},
    "thruster_2.a": {"type": "thruster", "base_p": 5.0, "amp_p": 1.1, "base_t": 370.0, "amp_t": 22.0, "freq": 0.05, "interval": (1.5, 4.0), "p_noise": 0.12, "t_noise": 1.0},
    "thruster_2.b": {"type": "thruster", "base_p": 5.1, "amp_p": 0.9, "base_t": 365.0, "amp_t": 18.0, "freq": 0.03, "interval": (2.5, 5.0), "p_noise": 0.14, "t_noise": 1.3},
    "thruster_2.c": {"type": "thruster", "base_p": 4.9, "amp_p": 1.3, "base_t": 375.0, "amp_t": 28.0, "freq": 0.07, "interval": (1.2, 3.2), "p_noise": 0.16, "t_noise": 1.8},
    "thruster_3.a": {"type": "thruster", "base_p": 3.8, "amp_p": 0.8, "base_t": 310.0, "amp_t": 15.0, "freq": 0.04, "interval": (2.0, 4.5), "p_noise": 0.10, "t_noise": 0.9},
    "thruster_3.b": {"type": "thruster", "base_p": 3.9, "amp_p": 0.7, "base_t": 315.0, "amp_t": 14.0, "freq": 0.05, "interval": (1.8, 4.0), "p_noise": 0.11, "t_noise": 0.8},
    "thruster_3.c": {"type": "thruster", "base_p": 4.0, "amp_p": 0.9, "base_t": 320.0, "amp_t": 16.0, "freq": 0.06, "interval": (2.2, 4.8), "p_noise": 0.13, "t_noise": 1.1},
    "oxygen_tank_1": {"type": "gas_valve", "base_p": 15.0, "amp_p": 3.0, "base_t": 220.0, "amp_t": 10.0, "freq": 0.02, "interval": (3.0, 7.0), "p_noise": 0.35, "t_noise": 0.7},
    "oxygen_tank_2": {"type": "gas_valve", "base_p": 14.5, "amp_p": 2.8, "base_t": 218.0, "amp_t": 9.0, "freq": 0.02, "interval": (3.5, 8.0), "p_noise": 0.30, "t_noise": 0.6},
    "hydrogen_tank_1": {"type": "gas_valve", "base_p": 22.0, "amp_p": 4.0, "base_t": 205.0, "amp_t": 12.0, "freq": 0.015, "interval": (4.0, 9.0), "p_noise": 0.45, "t_noise": 0.8},
    "hydrogen_tank_2": {"type": "gas_valve", "base_p": 21.5, "amp_p": 3.8, "base_t": 203.0, "amp_t": 11.0, "freq": 0.015, "interval": (4.5, 9.5), "p_noise": 0.40, "t_noise": 0.7},
}

CITIES = ["Miami", "Berlin", "Tokyo", "Cape Canaveral", "Sydney", "Houston", "Kourou"]

# In-memory persistent history per satellite
LOG_HISTORY: Dict[str, List[dict]] = {sat: [] for sat in SATELLITES_DB}
ERROR_LOGS: List[int] = []
NEXT_EMISSION: Dict[str, Dict[str, float]] = {}

MAX_HISTORY_PER_SAT = 3000

def get_next_interval(sensor_name: str) -> float:
    spec = SENSOR_SPECS.get(sensor_name, {})
    interval_range = spec.get("interval", (2.0, 5.0))
    return random.uniform(interval_range[0], interval_range[1])

def generate_measurement(sensor_name: str, sat_name: str, dt: datetime.datetime) -> dict:
    """Generiert realistisch rauschbehaftete Sensor-Messwerte mit gelegentlichen Peaks & Aussetzern."""
    spec = SENSOR_SPECS.get(sensor_name, {
        "type": "thruster", "base_p": 5.0, "amp_p": 1.0, "base_t": 300.0, "amp_t": 20.0, "freq": 0.05,
        "p_noise": 0.15, "t_noise": 1.0
    })
    sat_info = SATELLITES_DB.get(sat_name, {
        "name": sat_name,
        "model": "Standard",
        "launch_date": "20.11.1998",
        "nation": "International",
        "base_height": 400.0
    })
    
    t_sec = dt.timestamp()
    freq = spec["freq"]
    
    p_wave = math.sin(t_sec * freq)
    t_wave = math.cos(t_sec * freq * 0.8)
    h_wave = math.sin(t_sec * 0.01)
    
    noise_p = random.gauss(0, spec.get("p_noise", 0.15))
    noise_t = random.gauss(0, spec.get("t_noise", 1.0))
    noise_h = random.uniform(-1.5, 1.5)
    
    spike_p = random.choice([-1.2, 1.8]) if random.random() < 0.03 else 0.0
    spike_t = random.choice([-4.5, 5.2]) if random.random() < 0.03 else 0.0
    
    city_idx = (int(t_sec / 300) + hash(sat_name)) % len(CITIES)
    precision_p = random.choice([2, 3, 4, 5, 6])
    precision_t = random.choice([1, 2, 3, 4])
    
    pressure = max(0.1, round(spec["base_p"] + spec["amp_p"] * p_wave + noise_p + spike_p, precision_p))
    temperature = max(0.0, round(spec["base_t"] + spec["amp_t"] * t_wave + noise_t + spike_t, precision_t))
    height = max(100.0, round(sat_info.get("base_height", 400.0) + 15.0 * h_wave + noise_h, 4))
    city = CITIES[city_idx]
    
    unix_ts = int(t_sec)
    
    return {
        "sensor_name": sensor_name,
        "pressure": pressure,
        "temperature": temperature,
        "position": {
            "city": city,
            "height": height
        },
        "specs": {
            "name": sat_name,
            "model": sat_info.get("model", "Standard"),
            "launch_date": sat_info.get("launch_date", "20.11.1998"),
            "sensors": spec["type"],
            "nation": sat_info.get("nation", "International")
        },
        "timestamp": unix_ts
    }

def seed_initial_history():
    """Erzeugt ungleichmäßig verteilte historische Telemetriedaten der letzten 1 Std. inklusive zufälligen Sensorausfällen."""
    now = datetime.datetime.now(datetime.timezone.utc)
    now_ts = now.timestamp()
    start_ts = now_ts - 3600
    
    for sat_name, sat_info in SATELLITES_DB.items():
        if sat_name not in NEXT_EMISSION:
            NEXT_EMISSION[sat_name] = {}
        for sensor_name in sat_info["sensors"]:
            curr_ts = start_ts + random.uniform(0.0, 3.0)
            while curr_ts <= now_ts:
                if random.random() > 0.12: # 12% Sensoraussetzer
                    dt = datetime.datetime.fromtimestamp(curr_ts, tz=datetime.timezone.utc)
                    measurement = generate_measurement(sensor_name, sat_name, dt)
                    LOG_HISTORY[sat_name].append(measurement)
                
                curr_ts += get_next_interval(sensor_name)
            
            NEXT_EMISSION[sat_name][sensor_name] = curr_ts
            
        LOG_HISTORY[sat_name].sort(key=lambda x: x["timestamp"])

    ERROR_LOGS.append(int(now_ts - 2700))
    ERROR_LOGS.append(int(now_ts - 720))

async def background_realtime_generator():
    """Prüft im Sekunden-Takt alle Sensoren und sendet unregelmäßig Messdaten mit sporadischen Aussetzern."""
    while True:
        await asyncio.sleep(1)
        now = datetime.datetime.now(datetime.timezone.utc)
        now_ts = now.timestamp()
        
        for sat_name, sat_info in SATELLITES_DB.items():
            for sensor_name in sat_info["sensors"]:
                next_ts = NEXT_EMISSION.get(sat_name, {}).get(sensor_name, 0.0)
                if now_ts >= next_ts:
                    if random.random() > 0.10: # 10% Ausfallchance
                        measurement = generate_measurement(sensor_name, sat_name, now)
                        LOG_HISTORY[sat_name].append(measurement)
                        if len(LOG_HISTORY[sat_name]) > MAX_HISTORY_PER_SAT:
                            LOG_HISTORY[sat_name] = LOG_HISTORY[sat_name][-MAX_HISTORY_PER_SAT:]
                    
                    NEXT_EMISSION[sat_name][sensor_name] = now_ts + get_next_interval(sensor_name)

@app.on_event("startup")
async def startup_event():
    seed_initial_history()
    asyncio.create_task(background_realtime_generator())

# --- REST Endpunkte ---

@app.get("/satellites", summary="1. GET /satellites")
def get_satellites():
    return {"names": list(SATELLITES_DB.keys())}

@app.get("/satellites/sensors", summary="4. GET /satellites/sensors")
def get_all_sensors():
    return {"sensor_names": list(SENSOR_SPECS.keys())}

@app.get("/satellites/{name}/sensors", summary="GET /satellites/{name}/sensors")
def get_satellite_sensors(name: str):
    if name not in SATELLITES_DB:
        raise HTTPException(status_code=404, detail=f"Satellite '{name}' not found")
    return {"sensor_names": SATELLITES_DB[name]["sensors"]}

@app.get("/satellites/log/{amount}", summary="GET /satellites/log/{amount} (alle Satelliten)")
@app.get("/satellites/log", summary="GET /satellites/log (alle Satelliten)")
def get_all_satellites_log(amount: int = 100):
    all_logs = []
    for sat_name, logs in LOG_HISTORY.items():
        all_logs.extend(logs)
    
    all_logs.sort(key=lambda x: x["timestamp"])
    selected_logs = all_logs[-amount:] if amount else all_logs
    return {
        "amount": len(selected_logs),
        "data": selected_logs
    }

@app.get("/satellites/{name}", summary="2. GET /satellites/{name}")
def get_satellite_by_name(name: str):
    if name not in SATELLITES_DB:
        raise HTTPException(status_code=404, detail=f"Satellite '{name}' not found")
    return SATELLITES_DB[name]

@app.get("/satellites/{name}/log/{amount}", summary="3. GET /satellites/{name}/log/{amount}")
@app.get("/satellites/{name}/log", summary="3. GET /satellites/{name}/log")
def get_satellite_log(name: str, amount: int = 100):
    if name not in SATELLITES_DB:
        raise HTTPException(status_code=404, detail=f"Satellite '{name}' not found")
    
    logs = LOG_HISTORY[name]
    selected_logs = logs[-amount:] if amount else logs
    return {
        "amount": len(selected_logs),
        "data": selected_logs
    }

@app.get("/satellites/{name}/sensors/{sensor_name}/{amount}", summary="5. GET /satellites/{name}/sensors/{sensor_name}/{amount}")
@app.get("/satellites/{name}/sensors/{sensor_name}", summary="5. GET /satellites/{name}/sensors/{sensor_name}")
def get_satellite_sensor_log(name: str, sensor_name: str, amount: int = 100):
    if name not in SATELLITES_DB:
        raise HTTPException(status_code=404, detail=f"Satellite '{name}' not found")
    
    sat_info = SATELLITES_DB[name]
    if sensor_name not in sat_info["sensors"]:
        raise HTTPException(status_code=404, detail=f"Sensor '{sensor_name}' not found on satellite '{name}'")
    
    logs = [item for item in LOG_HISTORY[name] if item["sensor_name"] == sensor_name]
    selected_logs = logs[-amount:] if amount else logs
    
    return {
        "amount": len(selected_logs),
        "data": selected_logs
    }

@app.get("/error", summary="Optional: GET /error")
def get_errors():
    return {"timestamp": ERROR_LOGS}

if __name__ == "__main__":
    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=True)
