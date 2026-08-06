import React, { useEffect, useState } from "react";
import "./stylesheet.css";

function App() {
  const [suchbegriff, setSuchbegriff] = useState("");
  const [istOffen, setIstOffen] = useState(false);
  const [data, setData] = useState([]); // Start with an empty array

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch("http://127.0.0.1:8000/data/");

        if (!response.ok) {
          throw new Error("Fehler beim Laden der Daten");
        }

        const json = await response.json();
        console.log(json);
        setData(json);
      } catch (error) {
        console.error(error);
      }
    };

    fetchData();
  }, []);

  const gefilterteDaten = data.filter((item) =>
    item.name.toLowerCase().includes(suchbegriff.toLowerCase())
  );

  return (
    <>
      <button
        className="btn-top-right"
        onClick={() => setIstOffen(true)}
      >
        🔍 Suchen
      </button>

      {istOffen && (
        <div className="fullscreen-overlay">
          <button
            className="close-overlay-btn"
            onClick={() => {
              setIstOffen(false);
              setSuchbegriff("");
            }}
          >
            ✕ Schließen
          </button>

          <div className="overlay-content">
            <h2>Sensor Suche</h2>

            <input
              type="text"
              className="overlay-input"
              placeholder="Sensor Name eingeben..."
              value={suchbegriff}
              onChange={(e) => setSuchbegriff(e.target.value)}
              autoFocus
            />

            <div className="overlay-results">
              {suchbegriff !== "" && gefilterteDaten.length > 0 && (
                <ul>
                  {gefilterteDaten.map((item, index) => (
                    <li key={item.id ?? index}>
                      <strong>{item.name}</strong> — Temp: {item.temperature} |
                      Druck: {item.pressure}
                    </li>
                  ))}
                </ul>
              )}

              {suchbegriff !== "" && gefilterteDaten.length === 0 && (
                <p>Keine Ergebnisse gefunden.</p>
              )}
            </div>
          </div>
        </div>
      )}
    </>
  );
}

export default App;