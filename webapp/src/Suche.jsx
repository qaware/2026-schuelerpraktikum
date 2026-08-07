import React, { useState } from 'react';
import "./stylesheet.css";

function App() {
  const [suchbegriff, setSuchbegriff] = useState('');
  const [istOffen, setIstOffen] = useState(false);

  const daten = [
    { id: 1, name: "Thruster SC", pressure: "2.1 bar", temperature: "45 °C" },
    { id: 2, name: "Thruster North", pressure: "1.8 bar", temperature: "88 °C" },
    { id: 3, name: "Bodenstation Empfänger", pressure: "0.0 bar", temperature: "22 °C" }
  ];

  const gefilterteDaten = daten.filter((item) =>
    item.name.toLowerCase().includes(suchbegriff.toLowerCase())
  );

  return (
    <>
      {/* Der Button – bekommt die Klasse für Oben-Rechts */}
      <button 
        className="btn-top-right" 
        onClick={() => setIstOffen(true)}
      >
        🔍 Suchen
      </button>

      {/* Das Overlay – verdeckt bei Öffnung die GANZE Seite */}
      {istOffen && (
        <div className="fullscreen-overlay">
          <button 
            className="close-overlay-btn" 
            onClick={() => { setIstOffen(false); setSuchbegriff(''); }}
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
              {suchbegriff !== '' && gefilterteDaten.length > 0 && (
                <ul>
                  {gefilterteDaten.map((item) => (
                    <li key={item.id}>
                      <strong>{item.name}</strong> — Temp: {item.temperature} | Druck: {item.pressure}
                    </li>
                  ))}
                </ul>
              )}

              {suchbegriff !== '' && gefilterteDaten.length === 0 && (
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