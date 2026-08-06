import { useEffect, useRef } from "react";
import "./stylesheet_suche.css";

export default function Suche({
  suchbegriff,
  onChange,
  onSchliessen,
  trefferAnzahl,
  gesamtAnzahl,
  autoFokus = false,
}) {
  const inputRef = useRef(null);

  // Nach dem Wechsel von der Startseite direkt ins Suchfeld springen.
  useEffect(() => {
    if (autoFokus) inputRef.current?.focus();
  }, [autoFokus]);

  // Strg/Cmd+K fokussiert die Suche. Escape leert sie – und führt,
  // wenn schon leer, zurück zur Startseite.
  useEffect(() => {
    const handleKeyDown = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        inputRef.current?.focus();
      }
      if (e.key === "Escape") {
        if (suchbegriff !== "") {
          onChange("");
        } else {
          onSchliessen?.();
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onChange, onSchliessen, suchbegriff]);

  const istAktiv = suchbegriff.trim() !== "";

  return (
    <div className="suche">
      <div className="suche-feld">
        <span className="such-chip">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="11" cy="11" r="7" />
            <line x1="20" y1="20" x2="16.65" y2="16.65" />
          </svg>
        </span>

        <input
          ref={inputRef}
          type="search"
          className="suche-input"
          placeholder="Sensor, Typ oder Zeitstempel suchen…"
          value={suchbegriff}
          onChange={(e) => onChange(e.target.value)}
          aria-label="Sensordaten durchsuchen"
        />

        {istAktiv ? (
          <button
            type="button"
            className="suche-clear"
            onClick={() => {
              onChange("");
              inputRef.current?.focus();
            }}
            aria-label="Suche zurücksetzen"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <line x1="6" y1="6" x2="18" y2="18" />
              <line x1="18" y1="6" x2="6" y2="18" />
            </svg>
          </button>
        ) : (
          <kbd className="suche-kbd">⌘K</kbd>
        )}
      </div>

      <p className="suche-status" role="status">
        {istAktiv ? (
          <>
            <strong className="suche-treffer">{trefferAnzahl}</strong> von{" "}
            {gesamtAnzahl} Datensätzen
          </>
        ) : (
          `${gesamtAnzahl} Datensätze`
        )}
      </p>

      {!istAktiv && (
        <p className="suche-tipp">
          Tipp: mehrere Wörter kombinieren – z.&nbsp;B. <code>thruster 2026</code>
        </p>
      )}
    </div>
  );
}
