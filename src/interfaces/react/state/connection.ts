import type { OperationErrorCode } from "@/usecases/operationError";
type Base = { generation: number; graphEpoch: number; isSaving: boolean };
export type ConnectionState = Base & (
  { phase: "idle" | "checking" | "ready" } | { phase: "error"; error: OperationErrorCode }
);
export const initialConnectionState: ConnectionState = {
  phase: "idle", generation: 0, graphEpoch: 0, isSaving: false,
};
export type ConnectionAction =
  | { type: "checking"; generation: number; graphEpoch: number }
  | { type: "ready" | "cancelled"; generation: number }
  | { type: "failed"; generation: number; error: OperationErrorCode }
  | { type: "invalidated"; generation: number; graphEpoch: number; error: OperationErrorCode }
  | { type: "saveStarted"; graphEpoch: number }
  | { type: "saveFinished"; saved: boolean; generation: number; graphEpoch: number };
export function connectionReducer(state: ConnectionState, action: ConnectionAction): ConnectionState {
  switch (action.type) {
    case "checking": return { phase: "checking", generation: action.generation, graphEpoch: action.graphEpoch, isSaving: state.isSaving };
    case "ready":
    case "cancelled":
      if (action.generation !== state.generation) return state;
      return { phase: action.type === "ready" ? "ready" : "idle", generation: state.generation, graphEpoch: state.graphEpoch, isSaving: state.isSaving };
    case "failed":
      if (action.generation !== state.generation) return state;
      return { ...state, phase: "error", error: action.error };
    case "invalidated": return { ...state, phase: "error", error: action.error, generation: action.generation, graphEpoch: action.graphEpoch };
    case "saveStarted": return { ...state, isSaving: true, graphEpoch: action.graphEpoch };
    case "saveFinished": return action.saved
      ? { phase: "idle", generation: action.generation, graphEpoch: action.graphEpoch, isSaving: false }
      : { ...state, isSaving: false };
  }
}
