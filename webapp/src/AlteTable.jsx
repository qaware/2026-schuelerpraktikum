import "./stylesheet.css";
import { useEffect, useState } from "react";

function formatNumber(value) {
    if (value === null || value === undefined) return "–";
    return Number(value).toFixed(2);
}

/* Die Zeitstempel kommen in zwei Formaten aus dem Backend:
   "20260805_132116" und "2026-08-04T09:27:12.885375". */
function parseTime(time) {
    if (!time) return null;

    const match = /^(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})$/.exec(time);
    if (match) {
        const [, year, month, day, hour, minute, second] = match;
        return new Date(`${year}-${month}-${day}T${hour}:${minute}:${second}`);
    }

    const parsed = new Date(time);
    return isNaN(parsed.getTime()) ? null : parsed;
}

function formatTime(time) {
    const date = parseTime(time);
    if (!date) return time ?? "–";
    return date.toLocaleString("de-DE");
}

/* Sortiert nach Namen, wobei Zahlen als Zahlen verglichen werden.
   Dadurch kommt thruster_2.a vor thruster_10.a und nicht danach. */
function sortByName(sensors) {
    return [...sensors].sort((a, b) =>
        a.name.localeCompare(b.name, "de", { numeric: true })
    );
}

/* Neueste Messung zuerst. Eintraege ohne lesbare Zeit landen am Ende. */
function sortByTime(sensors) {
    return [...sensors].sort((a, b) => {
        const timeA = parseTime(a.time);
        const timeB = parseTime(b.time);
        if (!timeA) return 1;
        if (!timeB) return -1;
        return timeB - timeA;
    });
}

function SensorDetail({ sensorName, readings, onClose }) {
    // Fenster laesst sich auch mit der Escape-Taste schliessen
    useEffect(() => {
        const handleKeyDown = (event) => {
            if (event.key === "Escape") onClose();
        };
        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [onClose]);

    const sortedReadings = sortByTime(readings);

    return (
        <div className="modal-backdrop" onClick={onClose}>
            {/* Klick im Fenster soll es nicht gleich wieder schliessen */}
            <div className="modal" onClick={(event) => event.stopPropagation()}>
                <div className="modal-header">
                    <div>
                        <h2>{sensorName}</h2>
                        <p className="modal-subtitle">
                            {sortedReadings.length} Messungen
                        </p>
                    </div>
                    <button className="modal-close" onClick={onClose}>✕</button>
                </div>

                <div className="table-wrapper">
                    <table>
                        <thead>
                            <tr>
                                <th>Zeit</th>
                                <th className="num">Pressure</th>
                                <th className="num">Temperature</th>
                            </tr>
                        </thead>
                        <tbody>
                            {sortedReadings.map((reading, index) => (
                                <tr key={reading._id ?? index}>
                                    <td>{formatTime(reading.time)}</td>
                                    <td className="num">{formatNumber(reading.pressure)}</td>
                                    <td className="num">{formatNumber(reading.temperature)}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
}

function SensorSection({ title, sensors, onSelectSensor }) {
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
                                <td>
                                    <button
                                        className="sensor-name"
                                        onClick={() => onSelectSensor(sensor.name)}
                                    >
                                        {sensor.name}
                                    </button>
                                </td>
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
    const [allData, setAllData] = useState([]);
    const [selectedSensor, setSelectedSensor] = useState(null);

    const [thrusterData, setThrusterData] = useState([]);
    const [oxygenTankData, setOxygenTankData] = useState([]);
    const [hydrogenTankData, setHydrogenTankData] = useState([]);

    useEffect(() => {
        const fetchData = async () => {
            const response = await fetch("http://127.0.0.1:8000/data/");
            const json = await response.json();

            setAllData(json);
            setThrusterData(sortByName(json.filter((item) => item.type === "thruster")));
            setOxygenTankData(sortByName(json.filter((item) => item.type === "gas_valve" && item.name.startsWith("o"))));
            setHydrogenTankData(sortByName(json.filter((item) => item.type === "gas_valve" && item.name.startsWith("h"))));
        };
        fetchData();
    }, []);

    return (
        <>
            <SensorSection title="Thruster" sensors={thrusterData} onSelectSensor={setSelectedSensor} />
            <SensorSection title="Oxygen Tanks" sensors={oxygenTankData} onSelectSensor={setSelectedSensor} />
            <SensorSection title="Hydrogen Tanks" sensors={hydrogenTankData} onSelectSensor={setSelectedSensor} />

            {selectedSensor && (
                <SensorDetail
                    sensorName={selectedSensor}
                    readings={allData.filter((item) => item.name === selectedSensor)}
                    onClose={() => setSelectedSensor(null)}
                />
            )}
        </>
    );
}

export default SensorTable;
