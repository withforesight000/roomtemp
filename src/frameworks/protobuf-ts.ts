import { GraphData } from "@/domain/types";
import { TempgrpcdResponse } from "./tempgrpcd";

export function decodeIntoGraphData(buffer: ArrayBuffer): GraphData {
  const bytes = new Uint8Array(buffer);
  const message = TempgrpcdResponse.fromBinary(bytes);

  return new Map(Object.entries(message.ambientConditions)) as GraphData;
}
