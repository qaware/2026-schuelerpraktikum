import "./stylesheet.css";
import { useEffect, useState } from "react";

function formatNumber(value) {
    if (value === null || value === undefined) return "–";
    return Number(value).toFixed(2);
}

/* Sortiert nach Namen, wobei Zahlen als Zahlen verglichen werden.
   Dadurch kommt thruster_2.a vor thruster_10.a und nicht danach. */
function sortByName(sensors) {
    return [...sensors].sort((a, b) =>
        a.name.localeCompare(b.name, "de", { numeric: true })
    );
}

function SensorSection({ title, sensors }) {
    return (
        <section className="sensor-section">
            <h2>{title}</h2>
            <div className="table-wrapper">
                <table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th className="num">Pressure</th>
                            <th className="num">Temperature</th>
                        </tr>
                    </thead>
                    <tbody>
                        {sensors.length === 0 && (
                            <tr>
                                <td className="empty" colSpan={3}>Keine Daten vorhanden</td>
                            </tr>
                        )}

                        {sensors.map((sensor, index) => (
                            <tr key={sensor._id ?? index}>
                                <td>{sensor.name}</td>
                                <td className="num">{formatNumber(sensor.pressure)}</td>
                                <td className="num">{formatNumber(sensor.temperature)}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </section>
    );
}

function SensorTable() {
    const [thrusterData, setThrusterData] = useState([]);
    const [oxygenTankData, setOxygenTankData] = useState([]);
    const [hydrogenTankData, setHydrogenTankData] = useState([]);

    useEffect(() => {
        const fetchData = async () => {
            const response = await fetch("http://127.0.0.1:8000/data/");
            const json = await response.json();

            setThrusterData(sortByName(json.filter((item) => item.type === "thruster")));
            setOxygenTankData(sortByName(json.filter((item) => item.type === "gas_valve" && item.name.startsWith("o"))));
            setHydrogenTankData(sortByName(json.filter((item) => item.type === "gas_valve" && item.name.startsWith("h"))));
        };
        fetchData();
    }, []);

    return (
        <>
            <SensorSection title="Thruster" sensors={thrusterData} />
            <SensorSection title="Oxygen Tanks" sensors={oxygenTankData} />
            <SensorSection title="Hydrogen Tanks" sensors={hydrogenTankData} />
        </>
    );
}

export default SensorTable;
