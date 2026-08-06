import SensorTable from "./SensorTable";
import Suche from "./Suche";
import "./stylesheet.css";

function App() {
  return (
    <div>
      <h1>Satelliten Daten</h1>
      <Suche />
      <SensorTable />
    </div>
  );
}

export default App;