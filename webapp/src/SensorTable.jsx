import "./stylesheet.css";

function SensorTable() {

    const sensors = [
    {
      "name": "thruster_3.b", 
      "type": "thruster", 
      "pressure": 8.531939707650038, 
      "temperature": 413.2229067496139
    },
    {
      "name": "thruster_3.c",
      "type": "thruster",
      "pressure": 1.939630382365832,
      "temperature": 491.66510784087706
    }
  ];

    return (
        <table class="table">
            <thead>
                <tr>
                    <th class="border" >Name</th>
                    <th class="border" >Type</th>
                    <th class="border" >Pressure</th>
                    <th class="border" >Temperature</th>
                </tr>
            </thead>
        <tbody>

            {
                sensors.map(sensor => (

                    <tr>
                        <td class="border">{sensor.name}</td>
                        <td class="border">{sensor.type}</td>
                        <td class="border">{sensor.pressure}</td>
                        <td class="border">{sensor.temperature}</td>
                    </tr>

                ))
            }

        </tbody>

        </table>
    );
}

export default SensorTable;