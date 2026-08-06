/** Felder, die von der Suche durchsucht werden. */
const SUCHFELDER = ["name", "type", "time"];

/**
 * Filtert die Sensordaten. Mehrere Wörter werden UND-verknüpft,
 * d.h. "thruster 2026" findet nur Einträge, die beides enthalten.
 */
export function filterSensoren(daten, suchbegriff) {
  const begriffe = suchbegriff.toLowerCase().split(/\s+/).filter(Boolean);
  if (begriffe.length === 0) return daten;

  return daten.filter((sensor) => {
    const text = SUCHFELDER.map((feld) => sensor[feld] ?? "")
      .join(" ")
      .toLowerCase();
    return begriffe.every((begriff) => text.includes(begriff));
  });
}
