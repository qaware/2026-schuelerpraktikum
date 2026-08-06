import os
from datetime import datetime

import motor.motor_asyncio
from fastapi import FastAPI, HTTPException, status
from pymongo.errors import PyMongoError

from models import DataModel, GroupedDataModel, MeasurementModel


app = FastAPI()

MONGODB_URL = os.getenv(
    "MONGODB_URL",
    "mongodb://root:password@localhost:27017/"
)

client = motor.motor_asyncio.AsyncIOMotorClient(MONGODB_URL)

db = client["data"]

# Nur der jeweils neueste Messwert pro Sensor. Der alte Wert wird ueberschrieben.
current_collection = db["data"]

# Komplette Historie: alle Messwerte pro Sensor.
grouped_collection = db["data_wsi"]


def JSON_Transformation(
    data: DataModel
) -> GroupedDataModel:
    measurement = MeasurementModel(
        time=data.time,
        pressure=data.pressure,
        temperature=data.temperature,
    )

    return GroupedDataModel(
        type=data.type,
        name=data.name,
        measurements=[measurement],
    )


@app.post(
    "/data/",
    response_description="Receive and store data",
    status_code=status.HTTP_201_CREATED,
)
async def receive_data(data: DataModel):
    grouped_data = JSON_Transformation(data)

    measurements = [
        measurement.model_dump()
        for measurement in grouped_data.measurements
    ]

    sensor_key = {
        "type": grouped_data.type,
        "name": grouped_data.name,
    }

    try:
        # Historie: neue Messwerte hinten anhaengen, nichts loeschen.
        await grouped_collection.update_one(
            sensor_key,
            {
                "$push": {
                    "measurements": {
                        "$each": measurements
                    }
                },
            },
            upsert=True,
        )

        # Aktuelle Daten: den Messwert nur uebernehmen, wenn er neuer ist als
        # der gespeicherte. Sonst wuerde ein verspaetet eintreffender, aelterer
        # Messwert den aktuellen ueberschreiben.
        await current_collection.update_one(
            sensor_key,
            [
                {
                    "$set": {
                        "type": grouped_data.type,
                        "name": grouped_data.name,
                        "current": {
                            "$cond": [
                                {
                                    "$gt": [
                                        measurements[-1]["time"],
                                        {"$ifNull": ["$current.time", datetime.min]},
                                    ]
                                },
                                measurements[-1],
                                "$current",
                            ]
                        },
                    }
                }
            ],
            upsert=True,
        )
    except PyMongoError as error:
        raise HTTPException(
            status_code=503,
            detail=f"Datenbank nicht erreichbar: {error}",
        )

    return {
        "message": "MRS Data received and stored successfully.",
    }


@app.get(
    "/data/current",
    response_model=dict[str, dict[str, MeasurementModel]],
)
async def get_current_data():
    # Nur Dokumente lesen, die wirklich einen aktuellen Messwert haben.
    mongo_data = await current_collection.find(
        {"current": {"$exists": True}},
        {
            "_id": 0,
            "type": 1,
            "name": 1,
            "current": 1,
        },
    ).to_list(length=1000)

    if not mongo_data:
        raise HTTPException(
            status_code=404,
            detail="Keine Sensordaten vorhanden.",
        )

    result = {}

    for dataset in mongo_data:
        data_type = dataset["type"]
        name = dataset["name"]

        result.setdefault(data_type, {})
        result[data_type][name] = dataset["current"]

    return result


@app.get(
    "/data_wsi/{name}",
    response_model=dict[str, dict[str, list[MeasurementModel]]],
)
async def get_data_by_name(name: str):
    # Die Messwerte stehen in der Reihenfolge im Array, in der sie eingetroffen
    # sind. Sortiert nach Zeit ausliefern, damit ein Diagramm nicht zickzackt.
    mongo_data = await grouped_collection.aggregate(
        [
            {"$match": {"name": name}},
            {
                "$project": {
                    "_id": 0,
                    "type": 1,
                    "name": 1,
                    "measurements": {
                        "$sortArray": {
                            "input": "$measurements",
                            "sortBy": {"time": 1},
                        }
                    },
                }
            },
        ]
    ).to_list(length=1000)

    if not mongo_data:
        raise HTTPException(
            status_code=404,
            detail=f"Keine Sensordaten für '{name}' vorhanden.",
        )

    result = {}

    for dataset in mongo_data:
        data_type = dataset["type"]
        sensor_name = dataset["name"]

        result.setdefault(data_type, {})
        result[data_type][sensor_name] = dataset.get("measurements", [])

    return result
