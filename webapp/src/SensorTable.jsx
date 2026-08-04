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
        <table>
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Type</th>
                    <th>Pressure</th>
                    <th>Temperature</th>
                </tr>
            </thead>
        <tbody>

            {
                sensors.map(sensor => (

                    <tr>
                        <td>{sensor.name}</td>
                        <td>{sensor.type}</td>
                        <td>{sensor.pressure}</td>
                        <td>{sensor.temperature}</td>
                    </tr>

                ))
            }
            
        </tbody>

        </table>
    );
}

export default SensorTable;