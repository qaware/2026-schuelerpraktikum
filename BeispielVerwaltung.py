import os

import motor.motor_asyncio
from fastapi import FastAPI, HTTPException, status

from models import DataModel, GroupedDataModel, MeasurementModel


app = FastAPI()

MONGODB_URL = os.getenv(
    "MONGODB_URL",
    "mongodb://root:password@localhost:27017/"
)

client = motor.motor_asyncio.AsyncIOMotorClient(MONGODB_URL)

db = client["data"]
collection = db["grouped_data"]


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

    await collection.update_one(
        {
            "type": grouped_data.type,
            "name": grouped_data.name,
        },
        {
            "$setOnInsert": {
                "type": grouped_data.type,
                "name": grouped_data.name,
            },
            "$push": {
                "measurements": {
                    "$each": measurements
                }
            },
            "$set": {
                "current": measurements[-1]
            },
        },
        upsert=True,
    )

    return {
        "message": "Datensatz gespeichert"
    }


@app.get(
    "/data/current",
    response_model=dict[str, dict[str, MeasurementModel]],
)
async def get_current_data():
    mongo_data = await collection.find(
        {},
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