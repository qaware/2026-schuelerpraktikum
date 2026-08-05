import json
import pathlib
import requests
import os
import json

from models import DataModel, UpdateDataModel

def clean_filename(inp): # bereinigt Dateinamen von Dateipfad und -suffix
    temp_filename = list(inp)[8:-5]
    temp = ""
    for i in range(len(temp_filename)):
        temp += temp_filename[i]
    return temp


def validate_data(cont):
    data_dict = json.loads(cont)
    data_pairs = [[key, value] for key, value in data_dict.items()]
    if type(data_pairs[0][1]) == type(data_pairs[1][1]) == type(data_pairs[2][1]) == str and type(data_pairs[3][1]) == type(data_pairs[4][1]) == float:
        return True

    return False


def receive_sat_files():
    data_path = pathlib.Path("data")
    input_daten = []

    for item in data_path.iterdir(): # alle Daten im Ordner data
        if item.is_file():
            if item.suffix == ".json": # Datentyp prüfen
                with open(item, "r") as f: # öffnen zum Auslesen
                    json_content = f.readlines()[0]
                    json_content = list(json_content)
                        
                    filename = clean_filename(str(item))
                    if not json_content[0] == "{": # TODO: bessere Validierung über Dictionary
                        print("Fehlerhafte Datei gefunden: erstes Zeichen beginnt nicht mit {")
                    else:
                        datensatz = '{"time": "' + filename + '", '
                        for i in range(1, len(json_content)):
                            datensatz += json_content[i]

                        if validate_data(datensatz):
                            input_daten.append(datensatz)
                        else:
                            print("Fehlerhafte Datei gefunden: Datentypen stimmen nicht")

    
            else:
                print("Fehlerhafte Datei gefunden:", str(item.suffix))

        if str(item) != "data\Example_data.json":
            os.remove(item)

    return input_daten


sat_daten = receive_sat_files()
print(sat_daten)

for i in range(len(sat_daten)):
    response = requests.post("http://127.0.0.1:8000/data/", sat_daten[i], headers={"Content-Type": "application/json"})
    print(response, response.content)

# TODO: Einlesen und Formatierung von Dateien aus dem Ordner "data"
# TODO: Markierung, sodass Dateien nicht doppelt gelesen werden
# TODO: Wie gehen wir mit beschädigten oder fehlerhaften Dateien um?
# TODO: Welche Daten sind interessant für uns?
# TODO: Wie finden wir heraus, ob bereits Sensordaten existieren?
# TODO: Wie senden wir Dateien an die Verwaltung?
# TODO: ...

'''

if __name__ == '__main__':
    data = DataModel(name="Test")
    new_data = UpdateDataModel(name="Updated Test")
    answer1 = requests.post("http://127.0.0.1:8000/data/", data.json(), headers={"Content-Type": "application/json"})
    answer2 = requests.put(f"http://127.0.0.1:8000/data/{json.loads(answer1.content)['_id']}", new_data.json(), headers={"Content-Type": "application/json"})
    answer3 = requests.delete(f"http://127.0.0.1:8000/data/{json.loads(answer1.content)['_id']}")
    answer4 = requests.get(f"http://127.0.0.1:8000/data/{json.loads(answer1.content)['_id']}")
    print(answer1.content)
    print(answer2.content)
    print(answer3.content)
    print(answer4.content)

'''
