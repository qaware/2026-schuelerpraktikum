import { useCallback, useEffect, useMemo, useState } from "react";
import SensorTable from "./SensorTable";
import Startseite from "./Startseite";
import Suche from "./Suche";
import { filterSensoren } from "./suchfilter";
import { useSensorData } from "./useSensorData";
import "./stylesheet.css";

function durchschnitt(daten, feld) {
  const werte = daten.map((d) => d[feld]).filter((w) => typeof w === "number");
  if (werte.length === 0) return null;
  return werte.reduce((summe, wert) => summe + wert, 0) / werte.length;
}

function Kennzahl({ label, wert, einheit }) {
  return (
    <div className="kennzahl">
      <span className="kennzahl-label">{label}</span>
      <span className="kennzahl-wert">
        {wert ?? "–"}
        {wert != null && einheit && <span className="kennzahl-einheit">{einheit}</span>}
      </span>
    </div>
  );
}

/**
 * Eigene Komponente, damit der Sekundentakt nur diese Zeile neu rendert
 * und nicht die ganze Tabelle.
 */
function LetzteAktualisierung({ zeitstempel }) {
  const [jetzt, setJetzt] = useState(() => Date.now());

  useEffect(() => {
    const timer = setInterval(() => setJetzt(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);

  if (!zeitstempel) return null;

  const sekunden = Math.max(0, Math.round((jetzt - zeitstempel) / 1000));
  const text =
    sekunden < 5
      ? "gerade eben"
      : sekunden < 60
        ? `vor ${sekunden} s`
        : `vor ${Math.round(sekunden / 60)} min`;

  return <span className="aktualisiert">{text}</span>;
}

function App() {
  const { daten, status, fehler, laedtGerade, letzteAktualisierung, neuLaden } =
    useSensorData();
  const [ansicht, setAnsicht] = useState("start"); // "start" | "daten"
  const [suchbegriff, setSuchbegriff] = useState("");

  const gefilterteDaten = useMemo(
    () => filterSensoren(daten, suchbegriff),
    [daten, suchbegriff],
  );

  const zurStartseite = useCallback(() => {
    setAnsicht("start");
    setSuchbegriff("");
  }, []);

  const sucheZuruecksetzen = useCallback(() => setSuchbegriff(""), []);

  if (ansicht === "start") {
    return (
      <Startseite
        onSuchen={() => setAnsicht("daten")}
        anzahl={daten.length}
        status={status}
      />
    );
  }

  const schnittDruck = durchschnitt(gefilterteDaten, "pressure");
  const schnittTemperatur = durchschnitt(gefilterteDaten, "temperature");

  return (
    <div className="app">
      <header className="kopfzeile">
        <div className="kopfzeile-inhalt">
          <button
            type="button"
            className="marke marke--btn"
            onClick={zurStartseite}
            title="Zurück zur Startseite"
          >
            <svg className="marke-icon" viewBox="0 0 24 24" aria-hidden="true">
              <circle cx="12" cy="12" r="3.5" />
              <ellipse cx="12" cy="12" rx="10.5" ry="4.5" transform="rotate(-30 12 12)" />
            </svg>
            <span>Satelliten&nbsp;Daten</span>
          </button>

          <div className="kopfzeile-aktionen">
            <span className={`status-punkt status-punkt--${status}`}>
              {status === "ok" ? "Verbunden" : status === "laedt" ? "Lädt" : "Offline"}
            </span>
            <LetzteAktualisierung zeitstempel={letzteAktualisierung} />
            <button
              type="button"
              className={`btn btn--icon${laedtGerade ? " btn--laedt" : ""}`}
              onClick={neuLaden}
              disabled={laedtGerade}
            >
              <svg className="btn-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M20 11a8 8 0 1 0-2.3 5.7" />
                <polyline points="20 4 20 11 13 11" />
              </svg>
              Aktualisieren
            </button>
          </div>
        </div>
      </header>

      <main className="inhalt">
        <section className="sektion">
          <h1>Telemetrie</h1>
          <p className="untertitel">
            Live-Sensordaten der Bodenstation – durchsuchbar und sortierbar.
          </p>
        </section>

        <section className="kennzahlen">
          <Kennzahl label="Datensätze" wert={gefilterteDaten.length} />
          <Kennzahl label="Ø Druck" wert={schnittDruck?.toFixed(2)} einheit="bar" />
          <Kennzahl
            label="Ø Temperatur"
            wert={schnittTemperatur?.toFixed(1)}
            einheit="K"
          />
        </section>

        <Suche
          suchbegriff={suchbegriff}
          onChange={setSuchbegriff}
          onSchliessen={zurStartseite}
          trefferAnzahl={gefilterteDaten.length}
          gesamtAnzahl={daten.length}
          autoFokus
        />

        <SensorTable
          daten={gefilterteDaten}
          status={status}
          fehler={fehler}
          suchbegriff={suchbegriff}
          onZuruecksetzen={sucheZuruecksetzen}
        />
      </main>
    </div>
  );
}

export default App;
