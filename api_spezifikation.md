## API-Spezifikation

### Endpunkte

#### 1. `GET /satellites`
* **Response:**
  ```json
  {
    "names": ["str"]
  }
  ```

#### 2. `GET /satellites/{name}`
* **Response:**
  ```json
  {
    "name": "string",
    "model": "string",
    "launchdate": "timestamp",
    "sensors": ["string"],
    "nation": "string"
  }
  ```

#### 3. `GET /satellites/{name}/log`
* **Response / Meta:**
  * `amount`: `int`
  * Body Array: `[{ selber wie bei Datengenerierung }]`
* **Hinweis:** Sortiert nach `timestamp`

#### 4. `GET /satellites/sensors`
* **Response:**
  ```json
  {
    "sensor_names": ["str"]
  }
  ```

#### 5. `GET /satellites/{name}/sensors/{sensor_name}`
* **Response / Meta:**
  * `amount`: `int`
  * Body Array: `[{ selber wie bei Datengenerierung }]`

---

## Optional

#### `GET /error`
* **Response:**
  ```json
  {
    "timestamp": ["timestamp"]
  }
  ```
