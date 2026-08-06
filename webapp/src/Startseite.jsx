import { useEffect, useRef, useState } from "react";
import "./stylesheet.css";

/** Respektiert die Systemeinstellung "Bewegung reduzieren". */
function useReduzierteBewegung() {
  const [reduziert, setReduziert] = useState(
    () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );

  useEffect(() => {
    const abfrage = window.matchMedia("(prefers-reduced-motion: reduce)");
    const handler = (e) => setReduziert(e.matches);
    abfrage.addEventListener("change", handler);
    return () => abfrage.removeEventListener("change", handler);
  }, []);

  return reduziert;
}

/** Zählt weich auf den Zielwert hoch – auch bei späteren Änderungen. */
function useHochzaehlen(ziel, animieren) {
  const [wert, setWert] = useState(0);
  const vorherRef = useRef(0);

  useEffect(() => {
    const dauer = animieren ? 700 : 0;
    const von = vorherRef.current;
    const start = performance.now();
    let frame;

    const schritt = (jetzt) => {
      const t = dauer === 0 ? 1 : Math.min((jetzt - start) / dauer, 1);
      const weich = 1 - Math.pow(1 - t, 3);
      setWert(Math.round(von + (ziel - von) * weich));
      if (t < 1) {
        frame = requestAnimationFrame(schritt);
      } else {
        vorherRef.current = ziel;
      }
    };

    frame = requestAnimationFrame(schritt);
    return () => cancelAnimationFrame(frame);
  }, [ziel, animieren]);

  return wert;
}

export default function Startseite({ onSuchen, anzahl, status }) {
  const reduziert = useReduzierteBewegung();
  const angezeigteAnzahl = useHochzaehlen(anzahl, !reduziert);

  // ⌘K / Strg+K / Enter führen direkt in die Datenansicht
  useEffect(() => {
    const handleKeyDown = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        onSuchen();
      }
      if (e.key === "Enter") onSuchen();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onSuchen]);

  return (
    <div className="startseite">
      <div className="start-inhalt">
        {/* Kopfbereich mit der Suchleiste – sitzt oben */}
        <div className="start-kopf">
          <svg className="start-icon" viewBox="0 0 24 24" aria-hidden="true">
            <g transform="rotate(-30 12 12)">
              <path
                id="umlaufbahn"
                className="start-bahn"
                d="M 1.5 12 A 10.5 4.5 0 1 1 22.5 12 A 10.5 4.5 0 1 1 1.5 12"
              />
              {/* Kleiner Satellit, der die Umlaufbahn entlangzieht */}
              <circle className="start-satellit" r="1.15">
                {!reduziert && (
                  <animateMotion dur="7s" repeatCount="indefinite" rotate="auto">
                    <mpath href="#umlaufbahn" />
                  </animateMotion>
                )}
              </circle>
            </g>
            <circle className="start-planet" cx="12" cy="12" r="3.5" />
          </svg>

          <h1 className="start-titel">Satelliten Daten</h1>
          <p className="start-untertitel">
            Live-Telemetrie der Bodenstation – durchsuchbar und sortierbar.
          </p>

          <div className="start-suchbtn-wrap">
            <button type="button" className="start-suchbtn" onClick={onSuchen}>
              <span className="such-chip">
                <svg viewBox="0 0 24 24" aria-hidden="true">
                  <circle cx="11" cy="11" r="7" />
                  <line x1="20" y1="20" x2="16.65" y2="16.65" />
                </svg>
              </span>
              <span className="start-suchtext">Daten durchsuchen…</span>
              <kbd className="suche-kbd">⌘K</kbd>
            </button>
          </div>

          <p className="start-meta">
            <span className={`status-punkt status-punkt--${status}`}>
              {status === "ok" ? "Verbunden" : status === "laedt" ? "Lädt" : "Offline"}
            </span>
            {status === "ok" && (
              <>
                <span className="start-meta-trenner">·</span>
                <span>
                  <strong className="start-zahl">{angezeigteAnzahl}</strong> Datensätze
                  verfügbar
                </span>
              </>
            )}
          </p>
        </div>

        {/*
          Reservierter Bereich für die Tabelle, die später hier eingebaut wird.
          Zum Einsetzen einfach den Inhalt dieses <section> ersetzen –
          die Breite entspricht schon der Tabelle in der Datenansicht.
        */}
        <section className="start-tabellenplatz">
          <svg className="platzhalter-icon" viewBox="0 0 24 24" aria-hidden="true">
            <rect x="3" y="4" width="18" height="16" rx="2" />
            <line x1="3" y1="9.5" x2="21" y2="9.5" />
            <line x1="9" y1="9.5" x2="9" y2="20" />
          </svg>
          <p className="platzhalter-titel">Platz für die Datentabelle</p>
          <p className="platzhalter-text">Wird hier später eingefügt.</p>
        </section>
      </div>
    </div>
  );
}
