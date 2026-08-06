import { useMemo, useState } from "react";
import "./stylesheet.css";

/** Gültige Wertebereiche laut Bodenstation – Grundlage für die Mini-Balken. */
const BEREICHE = {
  pressure: { min: 0.5, max: 9, stellen: 2 },
  temperature: { min: 200, max: 500, stellen: 1 },
};

const SPALTEN = [
  { key: "name", label: "Name", typ: "text" },
  { key: "type", label: "Typ", typ: "text" },
  { key: "pressure", label: "Druck (bar)", typ: "zahl" },
  { key: "temperature", label: "Temperatur (K)", typ: "zahl" },
  { key: "time", label: "Zeitpunkt", typ: "zeit" },
];

const SKELETT_ZEILEN = [0, 1, 2, 3, 4];

/**
 * Die Bodenstation liefert zwei Zeitformate: "20260806_100132" (aus dem
 * Dateinamen) und ISO wie "2026-08-04T10:42:04.156". Beides wird zu einem
 * Date geparst – null, wenn nichts davon passt.
 */
function parseZeit(wert) {
  const treffer = /^(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})$/.exec(wert ?? "");
  if (treffer) {
    const [, jahr, monat, tag, stunde, minute, sekunde] = treffer;
    return new Date(jahr, monat - 1, tag, stunde, minute, sekunde);
  }
  const datum = new Date(wert ?? "");
  return Number.isNaN(datum.getTime()) ? null : datum;
}

/** Einheitliche Anzeige: "06.08.2026, 10:01:32". */
function formatZeit(wert) {
  const datum = parseZeit(wert);
  if (!datum) return wert ?? "–";
  return datum.toLocaleString("de-DE", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

/** Messwert mit kleinem Balken, der die Lage im gültigen Bereich zeigt. */
function Messwert({ wert, bereich }) {
  if (typeof wert !== "number") return <span className="leer">–</span>;

  const anteil = Math.min(
    Math.max((wert - bereich.min) / (bereich.max - bereich.min), 0),
    1,
  );

  return (
    <span
      className="messwert"
      title={`Gültiger Bereich: ${bereich.min} – ${bereich.max}`}
    >
      <span className="messwert-zahl">
        {wert.toLocaleString("de-DE", {
          minimumFractionDigits: bereich.stellen,
          maximumFractionDigits: bereich.stellen,
        })}
      </span>
      <span className="messwert-balken" aria-hidden="true">
        <span className="messwert-fuellung" style={{ width: `${anteil * 100}%` }} />
      </span>
    </span>
  );
}

/** Hebt den Suchtreffer im Text hervor. */
function Hervorhebung({ text, suchbegriff }) {
  const inhalt = String(text ?? "–");
  const begriffe = suchbegriff.toLowerCase().split(/\s+/).filter(Boolean);
  if (begriffe.length === 0) return inhalt;

  const muster = new RegExp(
    `(${begriffe.map((b) => b.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})`,
    "gi",
  );

  return inhalt.split(muster).map((teil, i) =>
    begriffe.includes(teil.toLowerCase()) ? <mark key={i}>{teil}</mark> : teil,
  );
}

function SensorTable({ daten, status, fehler, suchbegriff = "", onZuruecksetzen }) {
  const [sortierung, setSortierung] = useState({ spalte: "time", richtung: "ab" });

  const sortierteDaten = useMemo(() => {
    const spalte = SPALTEN.find((s) => s.key === sortierung.spalte);
    const faktor = sortierung.richtung === "auf" ? 1 : -1;

    return [...daten].sort((a, b) => {
      const x = a[sortierung.spalte];
      const y = b[sortierung.spalte];
      if (x == null) return 1;
      if (y == null) return -1;
      if (spalte?.typ === "zahl") return (x - y) * faktor;
      if (spalte?.typ === "zeit") {
        return ((parseZeit(x)?.getTime() ?? 0) - (parseZeit(y)?.getTime() ?? 0)) * faktor;
      }
      return String(x).localeCompare(String(y), "de") * faktor;
    });
  }, [daten, sortierung]);

  const sortieren = (key) =>
    setSortierung((alt) =>
      alt.spalte === key
        ? { spalte: key, richtung: alt.richtung === "auf" ? "ab" : "auf" }
        : { spalte: key, richtung: "auf" },
    );

  const laedt = status === "laedt";
  const leer = status === "ok" && sortierteDaten.length === 0;

  return (
    <div className="tabelle-karte">
      <table className="tabelle">
        <thead>
          <tr>
            {SPALTEN.map((spalte) => {
              const aktiv = sortierung.spalte === spalte.key;
              return (
                <th
                  key={spalte.key}
                  className={spalte.typ === "zahl" ? "rechts" : undefined}
                  aria-sort={
                    aktiv
                      ? sortierung.richtung === "auf"
                        ? "ascending"
                        : "descending"
                      : "none"
                  }
                >
                  <button
                    type="button"
                    className={`sort-btn${aktiv ? " sort-btn--aktiv" : ""}`}
                    onClick={() => sortieren(spalte.key)}
                    disabled={laedt}
                    title={`Nach ${spalte.label} sortieren`}
                  >
                    {spalte.label}
                    <span className="sort-pfeil">
                      {aktiv ? (sortierung.richtung === "auf" ? "↑" : "↓") : "↕"}
                    </span>
                  </button>
                </th>
              );
            })}
          </tr>
        </thead>

        <tbody>
          {laedt
            ? SKELETT_ZEILEN.map((i) => (
                <tr key={`skelett-${i}`} className="skelett-zeile">
                  {SPALTEN.map((spalte) => (
                    <td key={spalte.key}>
                      <span className="skelett" />
                    </td>
                  ))}
                </tr>
              ))
            : sortierteDaten.map((sensor) => (
                <tr key={sensor._id ?? `${sensor.name}-${sensor.time}`}>
                  <td className="zelle-name">
                    <Hervorhebung text={sensor.name} suchbegriff={suchbegriff} />
                  </td>
                  <td>
                    {sensor.type ? (
                      <span className="badge">
                        <Hervorhebung text={sensor.type} suchbegriff={suchbegriff} />
                      </span>
                    ) : (
                      <span className="leer">–</span>
                    )}
                  </td>
                  <td className="rechts">
                    <Messwert wert={sensor.pressure} bereich={BEREICHE.pressure} />
                  </td>
                  <td className="rechts">
                    <Messwert wert={sensor.temperature} bereich={BEREICHE.temperature} />
                  </td>
                  <td className="zeit">
                    <Hervorhebung
                      text={formatZeit(sensor.time)}
                      suchbegriff={suchbegriff}
                    />
                  </td>
                </tr>
              ))}
        </tbody>
      </table>

      {status === "fehler" && (
        <div className="tabelle-hinweis tabelle-hinweis--fehler">
          Verbindung zur Bodenstation fehlgeschlagen: {fehler}
        </div>
      )}

      {leer && (
        <div className="tabelle-hinweis">
          <svg className="hinweis-icon" viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="11" cy="11" r="7" />
            <line x1="20" y1="20" x2="16.65" y2="16.65" />
          </svg>
          {suchbegriff ? (
            <>
              <p className="hinweis-titel">Keine Treffer für „{suchbegriff}“</p>
              <p className="hinweis-text">
                Versuch es mit weniger Wörtern oder einem Teil des Namens.
              </p>
              <button type="button" className="btn" onClick={onZuruecksetzen}>
                Suche zurücksetzen
              </button>
            </>
          ) : (
            <>
              <p className="hinweis-titel">Noch keine Sensordaten empfangen</p>
              <p className="hinweis-text">
                Sobald die Bodenstation Daten sendet, erscheinen sie hier automatisch.
              </p>
            </>
          )}
        </div>
      )}
    </div>
  );
}

export default SensorTable;
