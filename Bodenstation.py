import json

import requests

from models import DataModel, UpdateDataModel

# TODO: Einlesen und Formatierung von Dateien aus dem Ordner "data"
# TODO: Markierung, sodass Dateien nicht doppelt gelesen werden
# TODO: Wie gehen wir mit beschädigten oder fehlerhaften Dateien um?
# TODO: Welche Daten sind interessant für uns?
# TODO: Wie finden wir heraus, ob bereits Sensordaten existieren?
# TODO: Wie senden wir Dateien an die Verwaltung?
# TODO: ...

if __name__ == '__main__':
    data = DataModel(name="Test")
    new_data = UpdateDataModel(name="Updated Test")
    answer1 = requests.post("http://127.0.0.1:8000/data/", data.model_dump_json(),
                             headers={"Content-Type": "application/json"})
    answer2 = requests.put(f"http://127.0.0.1:8000/data/{json.loads(answer1.content)['_id']}", new_data.model_dump_json(),
                            headers={"Content-Type": "application/json"})
    answer3 = requests.delete(f"http://127.0.0.1:8000/data/{json.loads(answer1.content)['_id']}")
    answer4 = requests.get(f"http://127.0.0.1:8000/data/{json.loads(answer1.content)['_id']}")
    print(answer1.content)
    print(answer2.content)
    print(answer3.content)
    print(answer4.content)
    