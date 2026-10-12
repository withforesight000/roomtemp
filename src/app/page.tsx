"use client";

import { useContext, useState } from "react";
import { useGraphData } from "@/interfaces/react/hooks/useGraphData";
import { GraphChart } from "@/interfaces/react/components/GraphChart";
import { Button } from "@/components/ui/button";
import { DateTimePicker24h } from "@/components/ui/expansions/datetime-picker";
import { GrpcRepoContext } from "@/interfaces/react/contexts/grpcRepoContext";
import { isWebDriverMockEnabled, MOCK_REFERENCE_TIME } from "@/mocks/webdriver";

export default function Home() {
  const { grpcRepo, state, connectionFailed } = useContext(GrpcRepoContext);
  const referenceTime = isWebDriverMockEnabled() ? MOCK_REFERENCE_TIME : Date.now();
  const [start, setStart] = useState<Date>(
    new Date(referenceTime - 6 * 60 * 60 * 1000)
  ); // 6 hours ago
  const [end, setEnd] = useState<Date>(new Date(referenceTime));
  const { data, fetch, loading, error, hasFetched, ready } = useGraphData(grpcRepo, state, connectionFailed);

  return (
    <div>
      <div className="flex flex-col">
        <div className="flex flex-row my-4 gap-4">
          <div className="flex-1 flex-column gap-2">
            <p>Start Time</p>
            <DateTimePicker24h
              value={start}
              onChange={(start) => setStart(start)}
            />
          </div>
          <div className="flex-1 flex-column gap-2">
            <p>End Time</p>
            <DateTimePicker24h value={end} onChange={(end) => setEnd(end)} />
          </div>
        </div>
        <Button onClick={() => fetch(start, end)} disabled={!ready || loading}>
          Fetch Data
        </Button>
        {error && <p role="alert" className="text-red-500">{error}</p>}
        {ready && hasFetched && data.length === 0 && <p>No data for the selected period</p>}
        {data.length > 0 && (
          <GraphChart
            key={`temperature-${data[0]?.time ?? "empty"}-${data[data.length - 1]?.time ?? "empty"}`}
            data={data}
            dataKey="temperature"
            description="Temperature (℃)"
          />
        )}
        {data.length > 0 && (
          <GraphChart
            key={`humidity-${data[0]?.time ?? "empty"}-${data[data.length - 1]?.time ?? "empty"}`}
            data={data}
            dataKey="humidity"
            description="Humidity (%)"
          />
        )}
        {data.length > 0 && (
          <GraphChart
            key={`illumination-${data[0]?.time ?? "empty"}-${data[data.length - 1]?.time ?? "empty"}`}
            data={data}
            dataKey="illumination"
            description="Illumination (lx)"
          />
        )}
      </div>
    </div>
  );
}
