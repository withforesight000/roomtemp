import { AmbientCondition, GraphData, Settings } from "@/domain/types";

const MOCK_SETTINGS: Settings = {
  url: "https://mock.grpc.example",
  accessToken: "test-token",
  useProxies: false,
  proxyUrl: "",
};

const WEBDRIVER_MOCK_FLAG =
  process.env.NEXT_PUBLIC_TAURI_WEBDRIVER_MOCKS === "1";

const now = Date.now();
const ambientPayload: Record<string, AmbientCondition> = {
  [`${now}-0`]: { temperature: 22.4, humidity: 48, illumination: 120 },
  [`${now + 60_000}-0`]: { temperature: 23.1, humidity: 47, illumination: 118 },
};

const GRAPH_DATA_MAP: GraphData = new Map(
  Object.entries(ambientPayload) as [string, AmbientCondition][]
);

const GRAPH_DATA_BUFFER = new TextEncoder().encode(
  JSON.stringify(ambientPayload)
).buffer;

export function isWebDriverMockEnabled(): boolean {
  return WEBDRIVER_MOCK_FLAG;
}

export function getMockSettings(): Settings {
  return { ...MOCK_SETTINGS };
}

export function getMockGraphData(): GraphData {
  return new Map(GRAPH_DATA_MAP);
}

export function getMockGraphBuffer(): ArrayBuffer {
  return GRAPH_DATA_BUFFER.slice(0);
}
