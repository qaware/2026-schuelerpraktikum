import "./stylesheet.css";
import { useEffect, useState } from "react";

function SensorTable() {
    const [data, setData] = useState(null);

    const [thrusterData, setThrusterData] = useState([]);
    const [oxygenTankData, setOxygenTankData] = useState([]);
    const [hydrogenTankData, setHydrogenTankData] = useState([]);

  useEffect(() => {
    const fetchData = async () => {
      const response = await fetch("http://127.0.0.1:8000/data/");
      const json = await response.json();
      console.log(JSON.stringify(json))
      setData(json);

      setThrusterData(json.filter((item) => item.type === "thruster"));
      setOxygenTankData(json.filter((item) => item.type === "gas_valve" && item.name.startsWith("o")));
      setHydrogenTankData(json.filter((item) => item.type === "gas_valve" && item.name.startsWith("h")));
    };
    fetchData();
  }, []);


    return (
        <>
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
                    thrusterData && thrusterData.map(sensor => (

                        
                        <tr>
                            <td>{sensor.name}</td>
                            <td>{sensor.pressure}</td>
                            <td>{sensor.temperature}</td>
                        </tr>
                    ))
                }

            </tbody>
            <h2>Oxygen Tanks</h2>
            </table>

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
                    oxygenTankData && oxygenTankData.map(sensor2 => (

                        
                        <tr>
                            <td>{sensor2.name}</td>
                            <td>{sensor2.pressure}</td>
                            <td>{sensor2.temperature}</td>
                        </tr>
                    ))
                }

            </tbody>
            </table>
            <h2>Hydrogen Tanks</h2>
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
                    hydrogenTankData && hydrogenTankData.map(sensor3 => (

                        
                        <tr>
                            <td>{sensor3.name}</td>
                            <td>{sensor3.pressure}</td>
                            <td>{sensor3.temperature}</td>
                        </tr>
                    ))
                }

            </tbody>
            </table>
    </>

        );
}

export default SensorTable;