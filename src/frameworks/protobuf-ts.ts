import { GraphData } from "@/domain/types";
import { GetAmbientConditionsResponse } from "@withforesight/tempgrpcd-protos";

export function decodeIntoGraphData(buffer: ArrayBuffer): GraphData {
  const bytes = new Uint8Array(buffer);
  const message = GetAmbientConditionsResponse.fromBinary(bytes);

  return new Map(Object.entries(message.ambientConditions)) as GraphData;
}
