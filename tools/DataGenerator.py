"""Telemetry data generator.

Produces satellite sensor measurements and pushes them to the backend ingest
endpoint. The signal shape (base + wave + gaussian noise + occasional spike),
the per-sensor emission intervals and the dropout rate mirror mock_api_server,
so the frontend behaves the same against either source.

One deliberate difference from the mock: tanks only carry one of the two
probes, so oxygen tanks report no temperature and hydrogen tanks no pressure.
Those fields go over the wire as null.
"""

import argparse
import datetime
import json
import math
import os
import os.path
import random
import time

import requests

BASE_PATH = os.path.dirname(os.path.dirname(__file__))

DEFAULT_INGEST_URL = os.environ.get("INGEST_URL", "http://localhost:8585/data")

TICK_SECONDS = 1.0
DROPOUT_CHANCE = 0.10

# Satellite catalog. A satellite keeps its identity across every measurement --
# model, nation, launch date and sensor list are looked up here, never re-rolled
# per record, otherwise /satellites/{name} would flicker between answers.
SATELLITES_DB: dict[str, dict] = {
    "ISS": {
        "name": "ISS",
        "model": "Zarya-1",
        "launch_date": "20.11.1998",
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
        "base_height": 1500000.0,
        "sensors": [
            "thruster_1.a", "thruster_1.b", "thruster_2.a",
            "oxygen_tank_1", "hydrogen_tank_1"
        ],
        "nation": "USA/ESA/CSA"
    }
}

# Baseline signal per sensor: carrier wave, noise amplitude and how often the
# sensor reports. "interval" is the (min, max) seconds between two emissions.
SENSOR_SPECS: dict[str, dict] = {
    "thruster_1.a": {"base_p": 4.5, "amp_p": 1.2, "base_t": 350.0, "amp_t": 25.0, "freq": 0.05, "interval": (1.5, 3.5), "p_noise": 0.15, "t_noise": 1.2},
    "thruster_1.b": {"base_p": 4.2, "amp_p": 1.0, "base_t": 340.0, "amp_t": 20.0, "freq": 0.04, "interval": (2.0, 4.0), "p_noise": 0.18, "t_noise": 1.5},
    "thruster_1.c": {"base_p": 4.8, "amp_p": 1.5, "base_t": 360.0, "amp_t": 30.0, "freq": 0.06, "interval": (1.0, 3.0), "p_noise": 0.20, "t_noise": 2.0},
    "thruster_2.a": {"base_p": 5.0, "amp_p": 1.1, "base_t": 370.0, "amp_t": 22.0, "freq": 0.05, "interval": (1.5, 4.0), "p_noise": 0.12, "t_noise": 1.0},
    "thruster_2.b": {"base_p": 5.1, "amp_p": 0.9, "base_t": 365.0, "amp_t": 18.0, "freq": 0.03, "interval": (2.5, 5.0), "p_noise": 0.14, "t_noise": 1.3},
    "thruster_2.c": {"base_p": 4.9, "amp_p": 1.3, "base_t": 375.0, "amp_t": 28.0, "freq": 0.07, "interval": (1.2, 3.2), "p_noise": 0.16, "t_noise": 1.8},
    "thruster_3.a": {"base_p": 3.8, "amp_p": 0.8, "base_t": 310.0, "amp_t": 15.0, "freq": 0.04, "interval": (2.0, 4.5), "p_noise": 0.10, "t_noise": 0.9},
    "thruster_3.b": {"base_p": 3.9, "amp_p": 0.7, "base_t": 315.0, "amp_t": 14.0, "freq": 0.05, "interval": (1.8, 4.0), "p_noise": 0.11, "t_noise": 0.8},
    "thruster_3.c": {"base_p": 4.0, "amp_p": 0.9, "base_t": 320.0, "amp_t": 16.0, "freq": 0.06, "interval": (2.2, 4.8), "p_noise": 0.13, "t_noise": 1.1},
    "oxygen_tank_1": {"base_p": 15.0, "amp_p": 3.0, "base_t": 220.0, "amp_t": 10.0, "freq": 0.02, "interval": (3.0, 7.0), "p_noise": 0.35, "t_noise": 0.7},
    "oxygen_tank_2": {"base_p": 14.5, "amp_p": 2.8, "base_t": 218.0, "amp_t": 9.0, "freq": 0.02, "interval": (3.5, 8.0), "p_noise": 0.30, "t_noise": 0.6},
    "hydrogen_tank_1": {"base_p": 22.0, "amp_p": 4.0, "base_t": 205.0, "amp_t": 12.0, "freq": 0.015, "interval": (4.0, 9.0), "p_noise": 0.45, "t_noise": 0.8},
    "hydrogen_tank_2": {"base_p": 21.5, "amp_p": 3.8, "base_t": 203.0, "amp_t": 11.0, "freq": 0.015, "interval": (4.5, 9.5), "p_noise": 0.40, "t_noise": 0.7},
}

CITIES: list[str] = [
    "Miami",
    "Berlin",
    "Tokyo",
    "Cape Canaveral",
    "Sydney",
    "Houston",
    "Kourou"
]


class Position:
    def __init__(self, city: str, height: float):
        self.city: str = city
        self.height: float = height


class Specs:
    def __init__(self, name: str, model: str, launch_date: str, sensors: list[str], nation: str):
        self.name: str = name
        self.model: str = model
        self.launch_date: str = launch_date
        self.sensors: list[str] = sensors
        self.nation: str = nation


class Sensor:
    """Sensor object, which stores all information of a given sensor."""

    def __init__(self, sensor_name: str, pressure: float | None = None, temperature: float | None = None, pos: Position | None = None, specs: Specs | None = None, timestamp: int | None = None):
        """Constructor"""

        self.sensor_name: str = sensor_name
        self.pressure: float | None = pressure
        self.temperature: float | None = temperature
        self.position: Position | None = pos
        self.specs: Specs | None = specs
        self.timestamp: int | None = timestamp

    def to_dict(self) -> dict:
        """Wire format. The backend rejects unknown fields, so this is spelled
        out explicitly rather than derived from __dict__."""
        return {
            "sensor_name": self.sensor_name,
            "pressure": self.pressure,
            "temperature": self.temperature,
            "position": self.position.__dict__ if self.position else None,
            "specs": self.specs.__dict__ if self.specs else None,
            "timestamp": self.timestamp,
        }


class DataGenerator:
    """Data Generator, which provides and stores sensor data of a given satellite."""

    def __init__(self):
        """Constructor"""
        now = time.time()

        # Per satellite, per sensor: the next timestamp at which that sensor is
        # due to report. Staggered so they do not all fire on the same tick.
        self.next_emission: dict[str, dict[str, float]] = {}
        for sat_name, sat_info in SATELLITES_DB.items():
            self.next_emission[sat_name] = {
                sensor_name: now + random.uniform(0.0, 3.0)
                for sensor_name in sat_info["sensors"]
            }

    @staticmethod
    def get_next_interval(sensor_name: str) -> float:
        spec = SENSOR_SPECS.get(sensor_name, {})
        interval_range = spec.get("interval", (2.0, 5.0))
        return random.uniform(interval_range[0], interval_range[1])

    def due_sensors(self, now_ts: float) -> list[tuple[str, str]]:
        """Returns every (satellite, sensor) pair whose emission time has passed
        and reschedules them. Rescheduling happens here rather than at the call
        site so a dropped measurement does not stall the sensor."""
        due: list[tuple[str, str]] = []

        for sat_name, sensors in self.next_emission.items():
            for sensor_name, next_ts in sensors.items():
                if now_ts >= next_ts:
                    due.append((sat_name, sensor_name))

        for sat_name, sensor_name in due:
            self.next_emission[sat_name][sensor_name] = now_ts + self.get_next_interval(sensor_name)

        return due

    @staticmethod
    def generate_measurement(sat_name: str, sensor_name: str, ts: float) -> Sensor:
        """Builds one noisy measurement for a sensor at a point in time."""
        spec = SENSOR_SPECS[sensor_name]
        sat_info = SATELLITES_DB[sat_name]

        freq = spec["freq"]
        p_wave = math.sin(ts * freq)
        t_wave = math.cos(ts * freq * 0.8)
        h_wave = math.sin(ts * 0.01)

        noise_p = random.gauss(0, spec["p_noise"])
        noise_t = random.gauss(0, spec["t_noise"])
        noise_h = random.uniform(-1.5, 1.5)

        spike_p = random.choice([-1.2, 1.8]) if random.random() < 0.03 else 0.0
        spike_t = random.choice([-4.5, 5.2]) if random.random() < 0.03 else 0.0

        pressure: float | None = max(0.1, round(spec["base_p"] + spec["amp_p"] * p_wave + noise_p + spike_p, 4))
        temperature: float | None = max(0.0, round(spec["base_t"] + spec["amp_t"] * t_wave + noise_t + spike_t, 2))
        height = max(100.0, round(sat_info["base_height"] + 15.0 * h_wave + noise_h, 4))

        city = CITIES[(int(ts / 300) + hash(sat_name)) % len(CITIES)]

        # Tanks only carry one of the two probes.
        if sensor_name.startswith("oxygen_tank"):
            temperature = None
        elif sensor_name.startswith("hydrogen_tank"):
            pressure = None

        return Sensor(
            sensor_name=sensor_name,
            pressure=pressure,
            temperature=temperature,
            timestamp=int(ts),
            pos=Position(city=city,
                         height=height),
            specs=Specs(name=sat_info["name"],
                        model=sat_info["model"],
                        launch_date=sat_info["launch_date"],
                        sensors=sat_info["sensors"],
                        nation=sat_info["nation"],
        ))

    @staticmethod
    def push_sensor_data(data: Sensor, url: str) -> bool:
        """POSTs one measurement to the backend. A backend restart must not take
        the generator down with it, so transport errors are logged and skipped."""
        try:
            response = requests.post(url, json=data.to_dict(), timeout=5)
        except requests.RequestException as exc:
            print(f"Push failed ({data.sensor_name}): {exc}")
            return False

        if response.status_code >= 400:
            print(f"Rejected ({data.sensor_name}): {response.status_code} {response.text.strip()}")
            return False

        return True

    @staticmethod
    def store_sensor_data(data: Sensor):
        """Writes a measurement to data/ as well. Off by default -- this grows
        without bound."""
        directory = os.path.join(BASE_PATH, "data")
        os.makedirs(directory, exist_ok=True)

        file_name = "TM_" + datetime.datetime.now().isoformat() + ".json"
        with open(os.path.join(directory, file_name), "w") as file:
            json.dump(data.to_dict(), file, indent=4)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Generate satellite telemetry and push it to the backend.")
    parser.add_argument("--url", default=DEFAULT_INGEST_URL, help="Backend ingest endpoint.")
    parser.add_argument("--store-files", action="store_true", help="Also write every measurement to data/ as JSON.")
    args = parser.parse_args()

    generator = DataGenerator()
    print(f"Pushing telemetry to {args.url}")

    while True:
        now_ts = time.time()

        for sat_name, sensor_name in generator.due_sensors(now_ts):
            if random.random() < DROPOUT_CHANCE:
                continue

            data = generator.generate_measurement(sat_name, sensor_name, now_ts)
            if generator.push_sensor_data(data, args.url):
                print(f"Sent {sat_name}/{sensor_name} @ {data.timestamp}")

            if args.store_files:
                generator.store_sensor_data(data)

        time.sleep(TICK_SECONDS)