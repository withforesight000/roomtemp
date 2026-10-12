export type GrpcErrorCode =
  | "invalid_configuration" | "authentication_failed" | "unavailable"
  | "timeout" | "not_serving" | "incompatible_service"
  | "not_connected" | "superseded" | "request_failed";

export type GrpcError = { code: GrpcErrorCode; message: string };

const messages: Record<GrpcErrorCode, string> = {
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

/** Uses safe, stable messages for IPC errors and deterministic mock failures. */
export function grpcError(code: GrpcErrorCode): GrpcError {
  return { code, message: messages[code] };
}

export function normalizeGrpcError(error: unknown): GrpcError {
  if (error && typeof error === "object" && "code" in error &&
      typeof error.code === "string" && Object.hasOwn(messages, error.code)) {
    return grpcError(error.code as GrpcErrorCode);
  }
  return grpcError("request_failed");
}

export function invalidatesConnection(error: GrpcError): boolean {
  return ["authentication_failed", "unavailable", "timeout", "not_connected"].includes(error.code);
}
