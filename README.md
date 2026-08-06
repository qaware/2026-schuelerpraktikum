# QAware Schülerpraktikum 2026

Repository für das Schülerpraktikum 2026 bei QAware.

## Installation

### 1. Create and activate a virtual environment for your python packages of this project:

```shell
python3 -m venv venv
source venv/bin/activate
```

If you created a virtual environment, set it as python interpreter in your IDE.

- In IntelliJ:
  - Click on File -> Project Structure.
  - Add a new Python SDK of type Virtualenv environment -> existing environment.
  - Set the interpreter to the python that is located in the directory of your newly created virtual environment.
- In VSCode:
  - Click on View -> Command Palette.
  - Search for Python: Select Interpreter.
  - Select the interpreter that is located in the directory of your newly created virtual environment.

### 2. Install all dependencies:

```shell
pip install -r requirements.txt
```

### 3. Start the database:

```shell
docker compose up -d
```

### 4. Start the uvicorn backend:

```shell
uvicorn BeispielVerwaltung:app --reload
```

The `--reload` command is used to be able to update the code and start the application automatically.

### 5. Optional: Add the database to your IDE

#### IntelliJ

- Select the database menu on the right of the window. 
- Add a new data source via the '+' icon. 
- Select MongoDB as type and give your database a name. 
- Enter the credentials from the [docker-compose.yml](./docker-compose.yml) file and test the connection.

![database_access.png](images/database_access.png)

#### VSCode

- Install the MongoDB for VSCode extension: <https://marketplace.visualstudio.com/items?itemName=mongodb.mongodb-vscode>
- Click on the MongoDB icon on the left of the window.
- Click on "Add Connection" and enter the credentials from the [docker-compose.yml](./docker-compose.yml) file.

## Usage

After the installation the API can be called via curl or the browser to serve the user with data or as data storage.

For example: <http://127.0.0.1:8000/hello_world>

A more generalized overview of existing APIs can be retrieved in a Swagger UI under:
<http://127.0.0.1:8000/docs>

## Helpful Links

- [AsyncIOMotorClient – Connection to MongoDB](https://motor.readthedocs.io/en/stable/api-asyncio/asyncio_motor_client.html)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Git Documentation](https://git-scm.com/docs)

## Maintainer

T. Prade, <thomas.prade@qaware.de>
R. Kalleicher, <robin.kalleicher@qaware.de>
T. Werner, <thomas.werner@qaware.de>

## Initial Code and Idea

R. Kalleicher, <robin.kalleicher@qaware.de>
C. Thelen, <christoph.thelen@qaware.de>
# PartyPizza
# PartyPizza
# PartyPizza
