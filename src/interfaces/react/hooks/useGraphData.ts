import { useCallback, useEffect, useReducer, useRef } from "react";
import type { FrontendOperations } from "@/usecases/frontendOperations";
import { invalidatesConnection, operationFailure } from "@/usecases/operationError";
import { presentGraphData } from "@/interfaces/presenters/graphPresenter";
import { presentOperationError } from "@/interfaces/presenters/errorPresenter";
import { graphReducer, initialGraphState } from "../state/graph";
import type { ConnectionSession } from "./useGrpcConnect";

export function useGraphData(operations: FrontendOperations, connection: ConnectionSession) {
  const [state, dispatch] = useReducer(graphReducer, initialGraphState);
  const request = useRef(0);
  const busy = useRef<number | null>(null);
  const { generation, graphEpoch, phase, isSaving } = connection.state;
  const { snapshot, connectionFailed } = connection;
  const ready = phase === "ready" && !isSaving;
  useEffect(() => {
    busy.current = null;
    dispatch({ type: "reset", generation, graphEpoch, request: ++request.current });
    return () => { request.current++; busy.current = null; };
  }, [generation, graphEpoch]);

  const fetch = useCallback(async (start: Date, end: Date) => {
    const session = snapshot();
    if (!session.active || !session.ready || session.saving || busy.current !== null ||
        session.generation !== generation || session.graphEpoch !== graphEpoch) return;
    const id = ++request.current;
    busy.current = id;
    const identity = { generation, graphEpoch, request: id };
    const active = () => {
      const latest = snapshot();
      return request.current === id && latest.active && latest.ready && !latest.saving &&
        latest.generation === generation && latest.graphEpoch === graphEpoch && latest.lifecycle === session.lifecycle;
    };
    dispatch({ type: "started", ...identity });
    try {
      const response = await operations.fetchGraphData(start, end);
      if (!active()) return;
      dispatch({ type: "succeeded", ...identity, data: presentGraphData(response) });
    } catch (error) {
      if (!active()) return;
      const failure = operationFailure(error);
      if (failure.code === "superseded") dispatch({ type: "cancelled", ...identity });
      else if (invalidatesConnection(failure)) connectionFailed(failure, generation);
      else dispatch({ type: "failed", ...identity, error: failure.code });
    } finally {
      if (busy.current === id) busy.current = null;
    }
  }, [operations, snapshot, generation, graphEpoch, connectionFailed]);
  const visible = ready && state.generation === generation && state.graphEpoch === graphEpoch;
  return {
    data: visible && state.phase === "success" ? state.data : [], fetch,
    loading: visible && state.phase === "loading", ready,
    error: visible && state.phase === "error" ? presentOperationError({ code: state.error }) : null,
    hasFetched: visible && state.phase === "success",
  };
}
