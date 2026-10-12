import { useCallback, useRef, useState } from "react";
import { GrpcRepository } from "@/interfaces/repositories/grpc";
import { GrpcError, normalizeGrpcError } from "@/domain/grpcError";

export type ConnectionState = {
  phase: "idle" | "checking" | "ready" | "error";
  status: string;
  generation: number;
  isSaving: boolean;
};

export const initialConnectionState: ConnectionState = {
  phase: "idle", status: "Disconnected", generation: 0, isSaving: false,
};

export function useGrpcConnect(grpcRepo: GrpcRepository) {
  const [state, setState] = useState(initialConnectionState);
  const current = useRef(state);
  const mounted = useRef(true);

  const update = useCallback((next: ConnectionState) => {
    current.current = next;
    if (mounted.current) setState(next);
  }, []);

  const connect = useCallback(async (): Promise<boolean> => {
    if (current.current.isSaving) return false;
    const generation = current.current.generation + 1;
    update({ phase: "checking", status: "Checking gRPC connection…", generation, isSaving: false });
    try {
      const status = await grpcRepo.connect();
      if (!mounted.current || current.current.generation !== generation) return false;
      update({ ...current.current, phase: "ready", status });
      return true;
    } catch (error: unknown) {
      if (!mounted.current || current.current.generation !== generation) return false;
      const failure = normalizeGrpcError(error);
      if (failure.code === "superseded") {
        update({ ...current.current, phase: "idle", status: "Disconnected" });
        return false;
      }
      update({ ...current.current, phase: "error", status: failure.message });
      return false;
    }
  }, [grpcRepo, update]);

  const beginSave = useCallback(() => {
    update({ ...current.current, isSaving: true });
  }, [update]);

  const finishSave = useCallback((saved: boolean) => {
    update(saved
      ? { phase: "idle", status: "Disconnected", generation: current.current.generation + 1, isSaving: false }
      : { ...current.current, isSaving: false });
  }, [update]);

  const connectionFailed = useCallback((error: GrpcError, generation: number) => {
    if (current.current.generation !== generation) return;
    update({ ...current.current, phase: "error", status: error.message, generation: generation + 1 });
  }, [update]);

  // The provider calls these around its startup effect, including Strict Mode remounts.
  const activate = useCallback(() => { mounted.current = true; }, []);
  const deactivate = useCallback(() => {
    mounted.current = false;
    current.current = { ...current.current, generation: current.current.generation + 1 };
  }, []);

  return { state, connect, beginSave, finishSave, connectionFailed, activate, deactivate };
}
