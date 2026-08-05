import "./stylesheet.css";
import { useEffect, useState } from "react";

function SensorTable() {
    const [data, setData] = useState(null);

  useEffect(() => {
    const fetchData = async () => {
      const response = await fetch("http://127.0.0.1:8000/data/");
      const json = await response.json();
      console.log(JSON.stringify(json))
      setData(json);
    };
    fetchData();
  }, []);


    return (
        <table>
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Pressure</th>
                    <th>Temperature</th>
                </tr>
            </thead>
        <tbody>

            {
                data && data.map(sensor => (

                    
                    <tr>
                        <td>{sensor.name}</td>
                        <td>{sensor.pressure}</td>
                        <td>{sensor.temperature}</td>
                    </tr>
                ))
            }

        </tbody>
        </table>
    );
}

export default SensorTable;