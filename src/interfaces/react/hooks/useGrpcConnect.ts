import { useCallback, useEffect, useReducer, useRef } from "react";
import type { FrontendOperations } from "@/usecases/frontendOperations";
import { type OperationFailure, operationFailure } from "@/usecases/operationError";
import { connectionReducer, initialConnectionState } from "../state/connection";

export type SessionSnapshot = {
  generation: number; graphEpoch: number; lifecycle: number;
  ready: boolean; saving: boolean; active: boolean;
};
export type SaveToken = { id: number; lifecycle: number };
export type ConnectionSession = ReturnType<typeof useGrpcConnect>;

export function useGrpcConnect(operations: FrontendOperations) {
  const [state, dispatch] = useReducer(connectionReducer, initialConnectionState);
  // This admission guard contains only operational identity/availability, never UI data or messages.
  const gate = useRef<SessionSnapshot>({ generation: 0, graphEpoch: 0, lifecycle: 0, ready: false, saving: false, active: true });
  const saveId = useRef(0);
  const snapshot = useCallback((): SessionSnapshot => ({ ...gate.current }), []);
  const connect = useCallback(async (): Promise<boolean> => {
    const session = gate.current;
    if (!session.active || session.saving) return false;
    const generation = ++session.generation;
    const lifecycle = session.lifecycle;
    session.ready = false;
    session.graphEpoch++;
    dispatch({ type: "checking", generation, graphEpoch: session.graphEpoch });
    const current = () => session.active && session.lifecycle === lifecycle && session.generation === generation;
    try {
      await operations.connect();
      if (!current()) return false;
      session.ready = true;
      dispatch({ type: "ready", generation });
      return true;
    } catch (error) {
      if (!current()) return false;
      const failure = operationFailure(error);
      dispatch(failure.code === "superseded" ? { type: "cancelled", generation }
        : { type: "failed", generation, error: failure.code });
      return false;
    }
  }, [operations]);

  const beginSave = useCallback((): SaveToken | null => {
    const session = gate.current;
    if (!session.active || session.saving) return null;
    session.saving = true;
    session.graphEpoch++;
    dispatch({ type: "saveStarted", graphEpoch: session.graphEpoch });
    return { id: ++saveId.current, lifecycle: session.lifecycle };
  }, []);
  const finishSave = useCallback((saved: boolean, token: SaveToken): boolean => {
    const session = gate.current;
    if (!session.active || session.lifecycle !== token.lifecycle || token.id !== saveId.current) return false;
    session.saving = false;
    if (saved) { session.generation++; session.graphEpoch++; session.ready = false; }
    dispatch({ type: "saveFinished", saved, generation: session.generation, graphEpoch: session.graphEpoch });
    return true;
  }, []);
  const connectionFailed = useCallback((error: OperationFailure, generation: number) => {
    const session = gate.current;
    if (!session.active || session.saving || session.generation !== generation) return;
    session.ready = false;
    session.generation++;
    session.graphEpoch++;
    dispatch({ type: "invalidated", generation: session.generation, graphEpoch: session.graphEpoch, error: error.code });
  }, []);
  useEffect(() => {
    gate.current.active = true;
    void connect();
    return () => {
      gate.current.active = false;
      gate.current.lifecycle++;
      gate.current.graphEpoch++;
    };
  }, [connect]);
  return { state, snapshot, connect, beginSave, finishSave, connectionFailed };
}
