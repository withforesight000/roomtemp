import { Page } from "@playwright/test";
const now = Date.now();
const payloadObj = {
  ambientConditions: {
    [`${now}-0`]: { temperature: 22.4, humidity: 48, illumination: 120 },
    [`${now + 60_000}-0`]: {
      temperature: 23.1,
      humidity: 47,
      illumination: 118,
    },
  },
};

const encoded = new TextEncoder().encode(JSON.stringify(payloadObj));
const serializedAmbientConditions = Array.from(encoded);

type TauriPayload = {
  cmd?: string;
  message?: { cmd?: string };
  __tauriModule?: string;
};

function resolveCommand(message: TauriPayload | string): string {
  if (typeof message === "string") return message;
  if (typeof message.cmd === "string") return message.cmd;
  if (message.message && typeof message.message.cmd === "string") {
    return message.message.cmd;
  }
  if (typeof message.__tauriModule === "string") return message.__tauriModule;
  return "";
}

export async function installTauriMocks(page: Page) {
  await page.addInitScript(
    ({ ambientPayload }) => {
      const ambientBuffer = Uint8Array.from(ambientPayload).buffer;

      // @ts-expect-error: injected for E2E only
      window.__TAURI_METADATA__ =
        // @ts-expect-error: injected for E2E only
        window.__TAURI_METADATA__ ?? {
          __windows: ["main"],
          __currentWindow: { label: "main" },
        };

      // @ts-expect-error: injected for E2E only
      window.__TAURI_IPC__ = async (message) => {
        const cmd = resolveCommand(message ?? {});
        switch (cmd) {
          case "connect_to_grpc_server":
            return "Mock connected to gRPC server";
          case "get_settings":
            return {
              url: "https://mock.grpc.example",
              accessToken: "test-token",
              useProxies: false,
              proxyUrl: "",
            };
          case "set_settings":
            return null;
          case "get_graph_data":
            return ambientBuffer;
          default:
            return null;
        }
      };
    },
    { ambientPayload: serializedAmbientConditions }
  );
}
