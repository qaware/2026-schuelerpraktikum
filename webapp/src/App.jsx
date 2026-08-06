import SensorTable from "./SensorTable";
import Suche from "./Suche";
import "./stylesheet.css";

function App() {
  return (
    <div>
      <h1>Satelliten Daten</h1>
      <Suche />
      <h2 className="text_left" >Thruster</h2>
      <SensorTable />
    </div>
  );
}

export default App;