import type { RechartPoint } from "@/interfaces/presenters/graphPresenter";
import type { OperationErrorCode } from "@/usecases/operationError";
type Identity = { generation: number; graphEpoch: number; request: number };
export type GraphState = Identity & (
  { phase: "idle" | "loading" } | { phase: "success"; data: RechartPoint[] } | { phase: "error"; error: OperationErrorCode }
);
export type GraphAction =
  | (Identity & { type: "reset" | "started" | "cancelled" })
  | (Identity & { type: "succeeded"; data: RechartPoint[] })
  | (Identity & { type: "failed"; error: OperationErrorCode });
export const initialGraphState: GraphState = { phase: "idle", generation: -1, graphEpoch: -1, request: 0 };
export function graphReducer(state: GraphState, action: GraphAction): GraphState {
  const identity = { generation: action.generation, graphEpoch: action.graphEpoch, request: action.request };
  if (action.type === "reset" || action.type === "started") {
    return { ...identity, phase: action.type === "reset" ? "idle" : "loading" };
  }
  if (state.generation !== action.generation || state.graphEpoch !== action.graphEpoch || state.request !== action.request) return state;
  switch (action.type) {
    case "cancelled": return { ...identity, phase: "idle" };
    case "succeeded": return { ...identity, phase: "success", data: action.data };
    case "failed": return { ...identity, phase: "error", error: action.error };
  }
}
