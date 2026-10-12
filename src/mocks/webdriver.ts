import { AmbientCondition, GraphData, Settings } from "@/domain/types";

const MOCK_SETTINGS: Settings = {
  url: "https://mock.grpc.example",
  accessToken: "test-token",
  useProxies: false,
  proxyUrl: "",
};

const WEBDRIVER_MOCK_FLAG =
  process.env.NEXT_PUBLIC_TAURI_WEBDRIVER_MOCKS === "1";

// Keep screenshots and date-picker assertions independent of the wall clock.
export const MOCK_REFERENCE_TIME = Date.parse("2026-10-01T12:00:00Z");
const now = MOCK_REFERENCE_TIME;
const ambientPayload: Record<string, AmbientCondition> = {
  [`${now}-0`]: { temperature: 22.4, humidity: 48, illumination: 120 },
  [`${now + 60_000}-0`]: { temperature: 23.1, humidity: 47, illumination: 118 },
};

const GRAPH_DATA_MAP: GraphData = new Map(
  Object.entries(ambientPayload) as [string, AmbientCondition][]
);

export function isWebDriverMockEnabled(): boolean {
  return WEBDRIVER_MOCK_FLAG;
}

export function getMockSettings(): Settings {
  failMockSettingsOperation("load");
  return { ...MOCK_SETTINGS };
}

export function getMockGraphData(): GraphData {
  return new Map(GRAPH_DATA_MAP);
}

export async function getMockGraphBuffer(): Promise<ArrayBuffer> {
  const { GetAmbientConditionsResponse } = await import("@withforesight/tempgrpcd-protos");
  const bytes = GetAmbientConditionsResponse.toBinary({ ambientConditions: ambientPayload });
  return Uint8Array.from(bytes).buffer;
}

// Fault injection is used only by repositories in explicitly enabled mock builds.
export function failMockSettingsOperation(operation: "load" | "save"): void {
  if (WEBDRIVER_MOCK_FLAG && typeof window !== "undefined" &&
      window.sessionStorage.getItem(`roomtemp.mock.settings.${operation}`) === "fail") {
    throw "Keystore: storage unavailable"; // Tauri serializes UIError as a string.
  }
}

export function recordMockConnection(): void {
  if (WEBDRIVER_MOCK_FLAG && typeof window !== "undefined") {
    const key = "roomtemp.mock.connections";
    const count = Number(window.sessionStorage.getItem(key) ?? "0");
    window.sessionStorage.setItem(key, String(count + 1));
  }
}
