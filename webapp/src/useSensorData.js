import { useCallback, useEffect, useState } from "react";

const API_URL = "http://127.0.0.1:8000/data/";
const POLL_INTERVALL = 10000; // alle 10s neu laden, da laufend Daten reinkommen

export function useSensorData() {
  const [daten, setDaten] = useState([]);
  const [status, setStatus] = useState("laedt"); // "laedt" | "ok" | "fehler"
  const [fehler, setFehler] = useState(null);
  const [laedtGerade, setLaedtGerade] = useState(false);
  const [letzteAktualisierung, setLetzteAktualisierung] = useState(null);

  const laden = useCallback(async (signal) => {
    setLaedtGerade(true);
    try {
      const response = await fetch(API_URL, { signal });
      if (!response.ok) {
        throw new Error(`Server antwortete mit ${response.status}`);
      }
      const json = await response.json();
      setDaten(Array.isArray(json) ? json : []);
      setStatus("ok");
      setFehler(null);
      setLetzteAktualisierung(Date.now());
    } catch (error) {
      if (error.name === "AbortError") return;
      setStatus("fehler");
      setFehler(error.message);
    } finally {
      setLaedtGerade(false);
    }
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    let timer;

    // Erst laden, dann den nächsten Abruf planen – so überholen sich
    // langsame Anfragen nicht gegenseitig.
    const planen = (verzoegerung) => {
      timer = setTimeout(async () => {
        await laden(controller.signal);
        if (!controller.signal.aborted) planen(POLL_INTERVALL);
      }, verzoegerung);
    };
    planen(0);

    return () => {
      controller.abort();
      clearTimeout(timer);
    };
  }, [laden]);

  // ohne Argument aufrufen – sonst landet z.B. das Klick-Event als Signal im fetch
  const neuLaden = useCallback(() => laden(), [laden]);

  return { daten, status, fehler, laedtGerade, letzteAktualisierung, neuLaden };
}
