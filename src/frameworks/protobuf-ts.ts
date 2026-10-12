import type { AmbientCondition, GraphData } from "@/domain/types";
import { OperationFailure } from "@/usecases/operationError";

function isCondition(value: unknown): value is AmbientCondition {
  return !!value && typeof value === "object" &&
    "temperature" in value && typeof value.temperature === "number" && Number.isFinite(value.temperature) &&
    "humidity" in value && typeof value.humidity === "number" && Number.isFinite(value.humidity) &&
    "illumination" in value && typeof value.illumination === "number" && Number.isFinite(value.illumination);
}
function graphData(value: unknown): GraphData {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new OperationFailure("request_failed");
  const entries: [string, AmbientCondition][] = [];
  for (const [key, condition] of Object.entries(value)) {
    const timestamp = key.split("-")[0];
    const time = Number(timestamp);
    if (!timestamp.trim() || !Number.isFinite(time) || !Number.isFinite(new Date(time).getTime()) || !isCondition(condition)) {
      throw new OperationFailure("request_failed");
    }
    entries.push([key, condition]);
  }
  return new Map(entries);
}

export async function decodeIntoGraphData(buffer: ArrayBuffer): Promise<GraphData> {
  try {
    // Keep generated bindings out of server-side eager initialization.
    const { GetAmbientConditionsResponse } = await import("@withforesight/tempgrpcd-protos");
    return graphData(GetAmbientConditionsResponse.fromBinary(new Uint8Array(buffer)).ambientConditions);
  } catch {
    // Preserve the existing legacy JSON-map compatibility path.
  }
  try {
    return graphData(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(buffer)));
  } catch {
    throw new OperationFailure("request_failed");
  }
}
