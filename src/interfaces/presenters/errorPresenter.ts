import type { OperationErrorCode } from "@/usecases/operationError";
const messages: Record<OperationErrorCode, string> = {
  invalid_configuration: "Check the URL, access token, TLS and proxy settings",
  authentication_failed: "Authentication failed. Check the access token and server access policy",
  unavailable: "The gRPC service is unavailable",
  timeout: "The gRPC request timed out",
  not_serving: "The gRPC service is not serving",
  incompatible_service: "The server does not support the required health check",
  not_connected: "Check the gRPC connection before fetching data",
  superseded: "This operation was superseded",
  request_failed: "The graph request failed",
};
export function presentOperationError(error: { code: OperationErrorCode }): string {
  return messages[error.code];
}
export function presentSettingsError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Settings could not be accessed";
}
