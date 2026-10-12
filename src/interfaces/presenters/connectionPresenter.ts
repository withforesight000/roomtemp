import type { ConnectionState } from "@/interfaces/react/state/connection";
import { presentOperationError } from "./errorPresenter";
export function presentConnection(state: ConnectionState, connectedMessage: string): string {
  if (state.isSaving) return "Saving settings…";
  switch (state.phase) {
    case "idle": return "Disconnected";
    case "checking": return "Checking gRPC connection…";
    case "ready": return connectedMessage;
    case "error": return presentOperationError({ code: state.error });
  }
}
