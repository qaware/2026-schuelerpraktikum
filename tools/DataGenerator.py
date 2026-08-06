import datetime
import json
import os.path
import random
import time
from xxlimited_35 import Null
import requests

BASE_PATH = path = os.path.dirname(os.path.dirname(__file__))

url = "http://localhost:8080/data"

class SensorKey:
    """Unique key of a sensor"""

    def __init__(self, name: str):
        """Constructor"""
        self.name: str = name


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

    def __init__(self, sensor_name: str, pressure: float | None = None, temperature: float | None = None, pos: Position |None = None, specs: Specs |None = None, zeit: float | None = None):
        """Constructor"""

        self.sensor_name: str = sensor_name
        self.pressure: float | None = pressure
        self.temperature: float | None = temperature
        self.position: Position | None= pos
        self.specs: Specs | None = specs
        self.zeit: float | None = zeit

class DataGenerator:
    """Data Generator, which provides and stores sensor data of a given satellite."""

    def __init__(self):
        """Constructor"""
        self.available_sensors: list[SensorKey] = [
            SensorKey(name="thruster_1.a"),
            SensorKey(name="thruster_1.b"),
            SensorKey(name="thruster_1.c"),
            SensorKey(name="thruster_2.a"),
            SensorKey(name="thruster_2.b"),
            SensorKey(name="thruster_2.c"),
            SensorKey(name="thruster_3.a"),
            SensorKey(name="thruster_3.b"),
            SensorKey(name="thruster_3.c"),
            SensorKey(name="oxygen_tank_1"),
            SensorKey(name="oxygen_tank_2"),
            SensorKey(name="hydrogen_tank_1"),
            SensorKey(name="hydrogen_tank_2")
        ]

    def generate_new_sensor_data(self):

        selected_key_idx = random.randint(0, len(self.available_sensors) - 1)

        selected_key = self.available_sensors[selected_key_idx]

       ## if (selected_key.name.startswith("thruster")):
         ##   return Sensor(sensor_name=selected_key.name)

        cities: list[str] = [
            "New York City",
            "Miami",
            "Mainz"
        ]
        city_idx = random.randint(0, len(cities) - 1)

        satellites: list[str] = [
            "Voyager",
            "Starlink",
            "Satellite1"
        ]
        satellite_idx = random.randint(0, len(satellites) - 1)

        models: list[str] = [
            "Communication",
            "Navigation"
        ]
        model_idx = random.randint(0, len(models) - 1)
        launch: list[str] = [
            "1223425787",
            "0823425767",
            "1253697879"
        ]
        launch_idx = random.randint(0, len(launch) - 1)
        sensors: list[str] = [
            "thruster_1.a",
            "thruster_1.b",
            "thruster_1.c",
            "thruster_2.a",
            "thruster_2.b",
            "thruster_2.c",
            "thruster_3.a",
            "thruster_3.b",
            "thruster_3.c",
            "oxygen_tank_1",
            "oxygen_tank_2",
            "hydrogen_tank_1",
            "hydrogen_tank_2"
        ]

        nations: list[str] = [
            "USA",
            "Russia",
            "Germany"
        ]

        nations_idx = random.randint(0, len(nations) - 1)

        pressure = random.uniform(0.5, 9.0)
        temperature = random.uniform(200.0, 500.0)
        height = random.uniform(50.0, 3000.0)
        zeit_str = int(time.time())






        print(selected_key.name)
        if selected_key.name in ("oxygen_tank_2", "oxygen_tank_1"):
            temperature = None
        elif selected_key.name in ("hydrogen_tank_2", "hydrogen_tank_1"):
            pressure = None

        sensor_data = Sensor(
            sensor_name=selected_key.name,
            pressure=pressure,
            temperature=temperature,
            zeit=zeit_str,
            pos=Position (city=cities[city_idx],
                          height=height),
            specs=Specs (name=satellites[satellite_idx],
                         model=models[model_idx],
                         launch_date=launch[launch_idx],
                         sensors=sensors,
                         nation=nations[nations_idx],
        ))

        return sensor_data



    @staticmethod

    def store_sensor_data(data: Sensor):



        content = data.__dict__
        file_name = "/data/TM_" + datetime.datetime.now().isoformat() + ".json"
        with open(BASE_PATH + file_name, "w") as file:
            json.dump(data, file, default=lambda o: o.__dict__, indent=4)









if __name__ == '__main__':
    generator = DataGenerator()

    while True:
        data = generator.generate_new_sensor_data()
        generator.store_sensor_data(data=data)
        print(f"Sucessfully stored: {data}")
        time.sleep(2)
