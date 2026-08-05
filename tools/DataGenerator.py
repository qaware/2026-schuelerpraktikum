import datetime
import json
import os.path
import random
import time
import numpy as np
from collections import Counter

BASE_PATH = path = os.path.dirname(os.path.dirname(__file__))


class SensorKey:
    """Unique key of a sensor"""

    def __init__(self, name: str, type: str):
        """Constructor"""
        self.name: str = name
        self.type: str = type


class Sensor:
    """Sensor object, which stores all information of a given sensor."""

    def __init__(self, name: str, type: str, pressure: float | None, temperature: float | None):
        """Constructor"""

        self.name: str = name
        self.type: str = type
        self.pressure: float | None = pressure
        self.temperature: float | None = temperature


class DataGenerator:
    """Data Generator, which provides and stores sensor data of a given satellite."""
    
    def __init__(self):
        """Constructor"""
        self.number=1
        self.available_sensors: list[SensorKey] = [
            SensorKey(name="thruster_1.a", type="thruster"),
            SensorKey(name="thruster_1.b", type="thruster"),
            SensorKey(name="thruster_1.c", type="thruster"),
            SensorKey(name="thruster_2.a", type="thruster"),
            SensorKey(name="thruster_2.b", type="thruster"),
            SensorKey(name="thruster_2.c", type="thruster"),
            SensorKey(name="thruster_3.a", type="thruster"),
            SensorKey(name="thruster_3.b", type="thruster"),
            SensorKey(name="thruster_3.c", type="thruster"),
            SensorKey(name="oxygen_tank_1", type="gas_valve"),
            SensorKey(name="oxygen_tank_2", type="gas_valve"),
            SensorKey(name="hydrogen_tank_1", type="gas_valve"),
            SensorKey(name="hydrogen_tank_2", type="gas_valve")
        ]

    def generate_new_sensor_data(self, result):
        self.result=result
        selected_key_idx = random.randint(0, len(self.available_sensors) - 1)
        selected_key = self.available_sensors[selected_key_idx]
        if result==2:
            pressure = random.uniform(1473829.123, 3847530.283)
            temperature = random.uniform(-220, -100)
        else:
            pressure = random.uniform(0.2, 0.5)
            temperature = random.uniform(200.0, 500.0)
        if result==3:
            sensor_data = Sensor(
                name="Error",
                type="Error",
                pressure="Error",
                temperature="Error"
                
            )
        else:
            sensor_data = Sensor(
                name=selected_key.name,
                type=selected_key.type,
                pressure=pressure,
                temperature=temperature
            
            )
    
        return sensor_data

    def store_sensor_data(self,result,data: Sensor):
        content = data.__dict__
        if result == 4:
            match self.number%4:
                case 0:
                    file_name="/data/TM_"+datetime.datetime.now().strftime("%Y%m%d_%H%M%S") + ".txt"
                case 1:
                    file_name="/data/TM_"+datetime.datetime.now().strftime("%Y%m%d_%H%M%S") + ".yaml"
                case 2:
                    file_name="/data/TM_"+datetime.datetime.now().strftime("%Y%m%d_%H%M%S") + ".xml"
                case 3:
                    file_name="/data/TM_"+datetime.datetime.now().strftime("%Y%m%d_%H%M%S") + ".pdf"
        else:
            file_name = "/data/TM_"+datetime.datetime.now().strftime("%Y%m%d_%H%M%S")+".json"
        with open(BASE_PATH + file_name, "w") as file:
            json.dump(content, file)
        self.number+=1

if __name__ == '__main__':

    generator = DataGenerator()
    list1=[]

    while True:
        result = np.random.choice([1,2,3,4], p=[0.6,0.1,0.1,0.2])
        data = generator.generate_new_sensor_data(result)
        generator.store_sensor_data(result,data=data)

        print(f"Sucessfully stored: {data}")
        list1.append(result)
        count=Counter(list1)
        time.sleep(3)
    print(count)
  