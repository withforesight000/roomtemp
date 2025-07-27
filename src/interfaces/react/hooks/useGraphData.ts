import { useState, useCallback } from "react";

import { FetchGraphData } from "@/usecases/fetchGraphData";
import { GrpcRepositoryImpl } from "@/interfaces/repositories/grpc";
import {
  presentGraphData,
  RechartPoint,
} from "@/interfaces/presenters/graphPresenter";

export function useGraphData(grpcRepo: GrpcRepositoryImpl) {
  const [connectivityStatus, setConnectivityStatus] =
    useState<string>("Disconnected");
  const [data, setData] = useState<RechartPoint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchGraphData = new FetchGraphData(grpcRepo);

  const fetch = async (startDate: Date, endDate: Date) => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetchGraphData.execute(startDate, endDate);
      console.log("Fetched graph data:");
      setData(presentGraphData(response));
    } catch (err) {
      setError(`Failed to fetch graph data: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return { data, connectivityStatus, fetch, loading, error };
}
