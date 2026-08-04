import SensorTable from "./SensorTable";
import Suche from "./Suche";
import "./stylesheet.css";

function App() {
  return (
    <div>
      <h1>Sateliten Daten</h1>
      <SensorTable />
      <Suche />
    </div>
  );
}

export default App;