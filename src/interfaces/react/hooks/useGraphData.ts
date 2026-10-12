import { useCallback, useEffect, useReducer, useRef } from "react";
import type { FrontendOperations } from "@/usecases/frontendOperations";
import { invalidatesConnection, operationFailure } from "@/usecases/operationError";
import { presentGraphData } from "@/interfaces/presenters/graphPresenter";
import { presentOperationError } from "@/interfaces/presenters/errorPresenter";
import { graphReducer, initialGraphState } from "../state/graph";
import type { ConnectionSession } from "./useGrpcConnect";

export function useGraphData(
  operations: Pick<FrontendOperations, "fetchGraphData">,
  connection: Pick<ConnectionSession, "state" | "snapshot" | "connectionFailed">,
) {
  const [state, dispatch] = useReducer(graphReducer, initialGraphState);
  const request = useRef(0);
  // Reject duplicate calls before React renders the loading state.
  const busy = useRef<number | null>(null);
  const { generation, graphEpoch, phase, isSaving } = connection.state;
  const { snapshot, connectionFailed } = connection;
  const ready = phase === "ready" && !isSaving;

  useEffect(() => {
    busy.current = null;
    dispatch({ type: "reset", generation, graphEpoch, request: ++request.current });
    return () => {
      request.current++;
      busy.current = null;
    };
  }, [generation, graphEpoch]);

  const fetch = useCallback(async (start: Date, end: Date) => {
    const session = snapshot();
    const canFetch = session.active && session.ready && !session.saving &&
      busy.current === null && session.generation === generation && session.graphEpoch === graphEpoch;
    if (!canFetch) return;

    const id = ++request.current;
    busy.current = id;
    const identity = { generation, graphEpoch, request: id };
    const isCurrentRequest = () => {
      const latest = snapshot();
      return request.current === id && latest.active && latest.ready && !latest.saving &&
        latest.generation === generation && latest.graphEpoch === graphEpoch && latest.lifecycle === session.lifecycle;
    };

    dispatch({ type: "started", ...identity });
    try {
      const response = await operations.fetchGraphData(start, end);
      if (!isCurrentRequest()) return;
      dispatch({ type: "succeeded", ...identity, data: presentGraphData(response) });
    } catch (error) {
      const failure = operationFailure(error);
      // Backend invalidation outlives this graph view; the session checks its generation.
      if (invalidatesConnection(failure)) {
        connectionFailed(failure, generation);
        return;
      }
      if (!isCurrentRequest()) return;
      if (failure.code === "superseded") {
        dispatch({ type: "cancelled", ...identity });
      } else {
        dispatch({ type: "failed", ...identity, error: failure.code });
      }
    } finally {
      // An obsolete request must not release a newer request's admission guard.
      if (busy.current === id) busy.current = null;
    }
  }, [operations, snapshot, generation, graphEpoch, connectionFailed]);
  const visibleState = ready && state.generation === generation && state.graphEpoch === graphEpoch
    ? state : null;
  return {
    data: visibleState?.phase === "success" ? visibleState.data : [],
    fetch,
    loading: visibleState?.phase === "loading",
    ready,
    error: visibleState?.phase === "error" ? presentOperationError({ code: visibleState.error }) : null,
    hasFetched: visibleState?.phase === "success",
  };
}
