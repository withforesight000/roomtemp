export const operationErrorCodes = [
  "invalid_configuration", "authentication_failed", "unavailable", "timeout",
  "not_serving", "incompatible_service", "not_connected", "superseded", "request_failed",
] as const;
export type OperationErrorCode = typeof operationErrorCodes[number];

export class OperationFailure extends Error {
  constructor(readonly code: OperationErrorCode) { super(code); }
}
export function operationFailure(error: unknown): OperationFailure {
  return error instanceof OperationFailure ? error : new OperationFailure("request_failed");
}
export function invalidatesConnection(error: OperationFailure): boolean {
  return ["authentication_failed", "unavailable", "timeout", "not_connected"].includes(error.code);
}
