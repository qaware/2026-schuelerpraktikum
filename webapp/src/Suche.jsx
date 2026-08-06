import React, { useState, useEffect } from 'react';
import "./stylesheet.css";

export default function Suche() {
  const [suchbegriff, setSuchbegriff] = useState('');
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

  useEffect(() => {
    const handleKeyDown = (e) => {
      if (e.key === 'Escape') {
        setIstOffen(false);
        setSuchbegriff('');
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return (
    <div className="mini-search-wrapper">
      {!istOffen ? (
        /* Kleiner Trigger Button oben rechts */
        <button className="mini-trigger-btn" onClick={() => setIstOffen(true)}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
          <span>Suchen</span>
        </button>
      ) : (
        /* Kompakte Suchbox */
        <div className="mini-search-box">
          <div className="mini-input-row">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="#888" strokeWidth="2.5">
              <circle cx="11" cy="11" r="8"></circle>
              <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
            <input
              type="text"
              placeholder="Sensor suchen..."
              value={suchbegriff}
              onChange={(e) => setSuchbegriff(e.target.value)}
              autoFocus
            />
            <button className="mini-close-btn" onClick={() => { setIstOffen(false); setSuchbegriff(''); }}>
              ✕
            </button>
          </div>

          {/* Ergebnisse in kompakter Liste */}
          {suchbegriff !== '' && (
            <div className="mini-results-list">
              {gefilterteDaten.length > 0 ? (
                gefilterteDaten.map((item) => (
                  <div key={item.id} className="mini-result-item">
                    <span className="item-name">{item.name}</span>
                    <span className="item-stats">{item.temperature} · {item.pressure}</span>
                  </div>
                ))
              ) : (
                <div className="mini-no-results">Keine Treffer</div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}