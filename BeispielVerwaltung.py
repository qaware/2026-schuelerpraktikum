import json
import os

import motor.motor_asyncio
import requests
from fastapi import FastAPI
from fastapi.encoders import jsonable_encoder
from starlette import status
from starlette.responses import JSONResponse

from models import DataModel, UpdateDataModel

app = FastAPI()
os.environ["MONGODB_URL"] = "mongodb://root:password@localhost:27017/?retryWrites=true&w=majority"
client = motor.motor_asyncio.AsyncIOMotorClient(os.environ["MONGODB_URL"])
db = client.get_database("data")

# TODO: Welche Daten müssen zur Verfügung gestellt werden?
# TODO: Wie verhindern wir, dass beschädigte Daten in das System gelangen?
# TODO: Wie melden wir Nutzern zurück, dass keine Sensordaten vorhanden sind?
# TODO: Wie melden wir Nutzern, dass bereits ein entsprechendes Datenset existiert?
# TODO: Was gehört zur Verwaltung der Daten noch?
# TODO: Benötigen wir noch andere Schnittstellen für unsere Nutzer?


@app.get("/hello_world", response_description="Hello World")
def hello_world():
    response = "Hello World"
    return JSONResponse(status_code=status.HTTP_200_OK, content=response)


@app.post("/data/", response_description="Create Data", response_model=DataModel)
async def create_data(data: DataModel):
    new_data = await db["data"].insert_one(jsonable_encoder(data))
    created_data = await db["data"].find_one({"_id": new_data.inserted_id})
    return JSONResponse(status_code=status.HTTP_201_CREATED, content=created_data)


@app.get("/data/", response_description="List All Data", response_model=list[DataModel])
async def list_data():
    data = await db["data"].find().to_list(1000)
    return JSONResponse(status_code=status.HTTP_200_OK, content=data)


@app.get("/data/{id}", response_description="Read Data", response_model=DataModel)
async def read_data(id: str):
    data = await db["data"].find_one({"_id": id})
    return JSONResponse(status_code=status.HTTP_200_OK, content=data)


@app.put("/data/{id}", response_description="Update Data", response_model=DataModel)
async def update_data(id: str, update: UpdateDataModel):
    await db["data"].update_one({"_id": id}, {"$set": jsonable_encoder(update)})
    updated_data = await db["data"].find_one({"_id": id})

    return JSONResponse(status_code=status.HTTP_200_OK, content=updated_data)


@app.delete("/data/{id}", response_description="Delete Data")
async def delete_data(id: str):
    await db["data"].delete_one({"_id": id})
    return JSONResponse(status_code=status.HTTP_204_NO_CONTENT, content=None)

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
