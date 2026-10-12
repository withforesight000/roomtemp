import { useEffect, useRef, useState } from "react";
import { FetchGraphData } from "@/usecases/fetchGraphData";
import { GrpcRepository } from "@/interfaces/repositories/grpc";
import { presentGraphData, RechartPoint } from "@/interfaces/presenters/graphPresenter";
import { ConnectionState } from "./useGrpcConnect";
import { GrpcError, invalidatesConnection, normalizeGrpcError } from "@/domain/grpcError";

export function useGraphData(
  grpcRepo: GrpcRepository, connection: ConnectionState,
  connectionFailed: (_error: GrpcError, _generation: number) => void,
) {
  const [resultGeneration, setResultGeneration] = useState(-1);
  const [data, setData] = useState<RechartPoint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasFetched, setHasFetched] = useState(false);
  const request = useRef(0);
  const current = useRef(connection);
  current.current = connection;
  const ready = connection.phase === "ready" && !connection.isSaving;

  useEffect(() => {
    setResultGeneration(ready ? connection.generation : -1);
    request.current += 1;
    setData([]);
    setError(null);
    setHasFetched(false);
    setLoading(false);
    return () => { request.current += 1; };
  }, [connection.generation, ready]);

  const fetch = async (startDate: Date, endDate: Date) => {
    if (!ready || loading) return;
    const id = ++request.current;
    const generation = connection.generation;
    const active = () => request.current === id &&
      current.current.generation === generation &&
      current.current.phase === "ready" && !current.current.isSaving;
    setResultGeneration(generation);
    setLoading(true);
    setError(null);
    setData([]);
    setHasFetched(false);
    try {
      const response = await new FetchGraphData(grpcRepo).execute(startDate, endDate);
      if (!active()) return;
      setData(presentGraphData(response));
      setHasFetched(true);
    } catch (failure: unknown) {
      if (!active()) return;
      const err = normalizeGrpcError(failure);
      if (err.code === "superseded") return;
      setError(err.message);
      if (invalidatesConnection(err)) connectionFailed(err, generation);
    } finally {
      if (active()) setLoading(false);
    }
  };

  const visible = ready && resultGeneration === connection.generation;
  return { data: visible ? data : [], fetch, loading: visible && loading,
    error: visible ? error : null, hasFetched: visible && hasFetched, ready };
}
