import { useEffect, useState } from "react";

export function DataFetching() {
  const [data, setData] = useState(null);

  useEffect(() => {
    const fetchData = async () => {
      const response = await fetch("http://127.0.0.1:8000/data/");
      const json = await response.json();
      setData(json);
    };
    fetchData();
  }, []);

  return (
    <>
      <div>
        <p>{JSON.stringify(data)}</p>
      </div>
    </>
  );
}
