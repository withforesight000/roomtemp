"use client";

import React, { useContext, useState } from "react";
import { useGraphData } from "@/interfaces/react/hooks/useGraphData";
import { GraphChart } from "@/interfaces/react/components/GraphChart";
import { Button } from "@/components/ui/button";
import { DateTimePicker24h } from "@/components/ui/expansions/datetime-picker";
import { GrpcRepoContext } from "@/interfaces/react/contexts/grpcRepoContext";

export default function Home() {
  const { grpcRepo } = useContext(GrpcRepoContext);
  const [start, setStart] = useState<Date>(
    new Date(Date.now() - 6 * 60 * 60 * 1000)
  ); // 6 hours ago
  const [end, setEnd] = useState<Date>(new Date());
  const { data, fetch, loading, error } = useGraphData(grpcRepo);

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
        <Button onClick={() => fetch(start, end)} disabled={loading}>
          Fetch Data
        </Button>
        {error && <p className="text-red-500">{error}</p>}
        {data && (
          <GraphChart
            data={data}
            dataKey="temperature"
            description="Temperature (℃)"
          />
        )}
        {data && (
          <GraphChart
            data={data}
            dataKey="humidity"
            description="Humidity (%)"
          />
        )}
        {data && (
          <GraphChart
            data={data}
            dataKey="illumination"
            description="Illumination (lx)"
          />
        )}
      </div>
    </div>
  );
}
