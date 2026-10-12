import { OperationFailure, operationErrorCodes, type OperationErrorCode } from "@/usecases/operationError";

/** Discard raw IPC messages, which may contain internal details. */
export function mapIpcError(error: unknown): OperationFailure {
  if (error instanceof OperationFailure) return error;
  if (error && typeof error === "object" && "code" in error &&
      typeof error.code === "string" && operationErrorCodes.some(code => code === error.code)) {
    return new OperationFailure(error.code as OperationErrorCode);
  }
  return new OperationFailure("request_failed");
}
export async function grpcOperation<T>(operation: () => Promise<T>): Promise<T> {
  try { return await operation(); } catch (error) { throw mapIpcError(error); }
}
