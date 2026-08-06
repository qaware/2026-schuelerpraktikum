# Import 
import asyncio
import json

import pandas as pd
import streamlit as st
import websockets
from streamlit_autorefresh import st_autorefresh

diagramm_platzhalter = st.empty()

# Anzeigenamen der Komponenten fuer die Tabelle
komponenten_namen = {
    "oxygen_tank_1": "Sauerstofftank",
    "hydrogen_tank_1": "Wasserstofftank",
    "thruster_1.a": "Triebwerk 1.A"
}


def zeitreihe(eintraege, feld, beschriftung):
    """Baut aus den Rohdaten einen DataFrame mit lesbarer Zeitachse."""
    df = pd.DataFrame({
        "Zeit": pd.to_datetime([e["timestamp"] for e in eintraege], format="ISO8601"),
        beschriftung: [e.get(feld) for e in eintraege]
    })
    # Aelteste Werte zuerst, damit das Diagramm von links nach rechts laeuft
    return df.sort_values("Zeit").set_index("Zeit")


def aktueller_wert(eintraege, feld):
    """Liefert den neuesten Messwert einer Komponente."""
    if not eintraege:
        return None
    neuester = max(eintraege, key=lambda e: pd.to_datetime(e["timestamp"], format="ISO8601"))
    wert = neuester.get(feld)
    return round(wert, 2) if wert is not None else None


async def stream_daten():

    uri = "ws://localhost:8000/ws"

    async with websockets.connect(uri) as websocket:

        st.success("Erfolgreich mit Satelliten-Datenstrom verbunden!")

        # Endlosschleife: Solange die Verbindung steht, lauschen wir auf Daten
        while True:
            try:
                daten_raw = await websocket.recv()
            

                daten = json.loads(daten_raw)

                wasserstoff = daten.get("hydrogen_tank_1", [])
                sauerstoff = daten.get("oxygen_tank_1", [])
                thruster_1 = daten.get("thruster_1.a", [])

                with diagramm_platzhalter.container():
                    st.title("Satellitendaten")

                    # --- Tabelle ---
                    st.subheader("Aktuelle Messwerte")

                    tabellen_zeilen = []
                    for schluessel, anzeigename in komponenten_namen.items():
                        eintraege = daten.get(schluessel, [])
                        tabellen_zeilen.append({
                            "Komponente": anzeigename,
                            "Temperatur (K)": aktueller_wert(eintraege, "temperature"),
                            "Druck (bar)": aktueller_wert(eintraege, "pressure")
                        })

                    st.dataframe(
                        pd.DataFrame(tabellen_zeilen),
                        hide_index=True,
                        use_container_width=True
                    )
                    # ----------------

                    st.subheader("Live-Diagramme")
                    
                    # --- SAUERSTOFF ---
                    col1, col2 = st.columns(2)
                    
                    # Hier sind Spalte und Expander in einer Zeile kombiniert:
                    with col1, st.expander("Temperatur vom Sauerstofftank", expanded=True):
                        df_o_temp = zeitreihe(sauerstoff, "temperature", "Temperatur (K)")
                        st.line_chart(
                            df_o_temp,
                            x_label="Zeit (Uhrzeit)",
                            y_label="Temperatur (K)"
                        )

                    with col2, st.expander("Druck vom Sauerstofftank", expanded=True):
                        df_o_druck = zeitreihe(sauerstoff, "pressure", "Druck (bar)")
                        st.line_chart(
                            df_o_druck,
                            x_label="Zeit (Uhrzeit)",
                            y_label="Druck (bar)"
                        )

                    # --- WASSERSTOFF ---
                    col3, col4 = st.columns(2)
                    
                    with col3, st.expander("Temperatur vom Wasserstofftank", expanded=True):
                        df_h_temp = zeitreihe(wasserstoff, "temperature", "Temperatur (K)")
                        st.line_chart(
                            df_h_temp,
                            x_label="Zeit (Uhrzeit)",
                            y_label="Temperatur (K)"
                        )

                    with col4, st.expander("Druck vom Wasserstofftank", expanded=True):
                        df_h_druck = zeitreihe(wasserstoff, "pressure", "Druck (bar)")
                        st.line_chart(
                            df_h_druck,
                            x_label="Zeit (Uhrzeit)",
                            y_label="Druck (bar)"
                        )

                    # --- TRIEBWERK ---
                    col5, col6 = st.columns(2)
                    
                    with col5, st.expander("Temperatur vom Triebwerk 1.A", expanded=True):
                        df_t_temp = zeitreihe(thruster_1, "temperature", "Temperatur (K)")
                        st.line_chart(
                            df_t_temp,
                            x_label="Zeit (Uhrzeit)",
                            y_label="Temperatur (K)"
                        )

                    with col6, st.expander("Druck vom Triebwerk 1.A", expanded=True):
                        df_t_druck = zeitreihe(thruster_1, "pressure", "Druck (bar)")
                        st.line_chart(
                            df_t_druck,
                            x_label="Zeit (Uhrzeit)",
                            y_label="Druck (bar)"
                        )
                    
            except websockets.exceptions.ConnectionClosed:
                st.error("Verbindung zum Server verloren. Versuche erneuten Verbindungsaufbau...")
                await asyncio.sleep(2)
                break           



daten = asyncio.run(stream_daten())


  