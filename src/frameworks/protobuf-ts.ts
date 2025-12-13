import { GraphData } from "@/domain/types";

export async function decodeIntoGraphData(
  buffer: ArrayBuffer
): Promise<GraphData> {
  // First try the protobuf-ts generated types if available (best-effort).
  try {
    // dynamic import so server-side bundle doesn't fail when the package is absent
    const mod = await import("@withforesight/tempgrpcd-protos");
    const GetAmbientConditionsResponse = (mod as any)
      .GetAmbientConditionsResponse;
    if (
      GetAmbientConditionsResponse &&
      typeof GetAmbientConditionsResponse.fromBinary === "function"
    ) {
      const bytes = new Uint8Array(buffer);
      const message = GetAmbientConditionsResponse.fromBinary(bytes);
      return new Map(Object.entries(message.ambientConditions)) as GraphData;
    }
  } catch (err) {
    // ignore and fall back to JSON
  }

  // Fallback: try to parse buffer as a UTF-8 JSON string representing the ambientConditions map
  try {
    const s = new TextDecoder().decode(buffer);
    const obj = JSON.parse(s);
    return new Map(Object.entries(obj)) as GraphData;
  } catch (err) {
    // As a last resort, return an empty map
    return new Map();
  }
}
