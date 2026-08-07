from typing import Any

from bson import ObjectId
from pydantic import (
    BaseModel,
    ConfigDict,
    Field,
    GetCoreSchemaHandler,
    GetJsonSchemaHandler,
)
from pydantic.json_schema import JsonSchemaValue
from pydantic_core import core_schema

'''
class PyObjectId(ObjectId):
    @classmethod
    def __get_pydantic_core_schema__(
        cls,
        source_type: Any,
        handler: GetCoreSchemaHandler,
    ) -> core_schema.CoreSchema:
        return core_schema.no_info_plain_validator_function(
            cls.validate,
            serialization=core_schema.plain_serializer_function_ser_schema(str),
        )

    @classmethod
    def validate(cls, value: Any) -> ObjectId:
        if isinstance(value, ObjectId):
            return value

        if not ObjectId.is_valid(value):
            raise ValueError("Invalid ObjectId")

        return ObjectId(value)

    @classmethod
    def __get_pydantic_json_schema__(
        cls,
        schema: core_schema.CoreSchema,
        handler: GetJsonSchemaHandler,
    ) -> JsonSchemaValue:
        return {"type": "string"}


class DataModel(BaseModel):
    id: PyObjectId = Field(
        default_factory=PyObjectId,
        alias="_id",
    )

    time: str
    name: str
    type: str
    pressure: float
    temperature: float

    model_config = ConfigDict(
        populate_by_name=True,
        arbitrary_types_allowed=True,
        json_encoders={ObjectId: str},
        json_schema_extra={
            "example": {
                "time": "2026-08-05T13:30:00",
                "name": "thruster_1.a",
                "type": "thruster",
                "pressure": 1.1,
                "temperature": 200.0,
            }
        },
    )


class UpdateDataModel(BaseModel):
    time: str | None = None
    name: str | None = None
    type: str | None = None
    pressure: float | None = None
    temperature: float | None = None

    model_config = ConfigDict(
        json_schema_extra={
            "example": {
                "pressure": 1.2,
                "temperature": 205.0,
            }
        }
    )


class MeasurementModel(BaseModel):
    time: str
    pressure: float
    temperature: float


class GroupedDataModel(BaseModel):
    type: str
    name: str
    measurements: list[MeasurementModel]

    model_config = ConfigDict(
        json_schema_extra={
            "example": {
                "type": "thruster",
                "name": "thruster_1.a",
                "measurements": [
                    {
                        "time": "2026-08-05T13:30:00",
                        "pressure": 1.1,
                        "temperature": 200.0,
                    },
                    {
                        "time": "2026-08-05T13:35:00",
                        "pressure": 1.2,
                        "temperature": 205.0,
                    },
                ],
            }
        }
    )



class MeasurementModel(BaseModel):
    time: str
    pressure: float
    temperature: float


class DataModel(BaseModel):
    time: str
    name: str = Field(min_length=1)
    type: str = Field(min_length=1)
    pressure: float
    temperature: float


class GroupedDataModel(BaseModel):
    type: str
    name: str
    measurements: list[MeasurementModel]
'''
from datetime import datetime

from pydantic import BaseModel, Field


class DataModel(BaseModel):
    time: datetime
    name: str = Field(min_length=1)
    type: str = Field(min_length=1)
    pressure: float
    temperature: float


class MeasurementModel(BaseModel):
    time: datetime
    pressure: float
    temperature: float


class GroupedDataModel(BaseModel):
    type: str
    name: str
    measurements: list[MeasurementModel]