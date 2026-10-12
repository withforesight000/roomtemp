import { expect, test, mock } from "bun:test";
import { createFrontendOperations } from "@/usecases/frontendOperations";
import { mapIpcError } from "@/frameworks/ipcError";
import { presentOperationError } from "@/interfaces/presenters/errorPresenter";
import { TauriGrpcRepository } from "@/interfaces/repositories/grpc";
import { TauriSettingsRepository } from "@/interfaces/repositories/settings";
import { GetAmbientConditionsResponse } from "@withforesight/tempgrpcd-protos";

const calls: [string, unknown][] = [];
const settings = { url: "https://example.test", accessToken: "disposable", useProxies: false, proxyUrl: "" };
mock.module("@tauri-apps/api/core", () => ({ invoke: async (command: string, args: unknown) => {
  calls.push([command, args]);
  if (command === "get_graph_data") return Uint8Array.from(GetAmbientConditionsResponse.toBinary({ ambientConditions: {} })).buffer;
  if (command === "get_settings") return settings;
  return "backend response is not UI copy";
} }));
test("operations inject adapters and preserve IPC contracts", async () => {
  calls.length = 0;
  const grpc = new TauriGrpcRepository();
  const operations = createFrontendOperations(grpc, grpc, new TauriSettingsRepository());
  expect(await operations.connect()).toBeUndefined();
  expect(await operations.fetchGraphData(new Date(1000), new Date(2000))).toEqual(new Map());
  expect(await operations.loadSettings()).toEqual(settings);
  await operations.saveSettings(settings);
  expect(calls).toEqual([["connect_to_grpc_server", undefined], ["get_graph_data", { startTime: 1000, endTime: 2000 }], ["get_settings", undefined], ["set_settings", settings]]);
});
test("raw IPC details are discarded at the adapter boundary", () => {
  const error = mapIpcError({ code: "authentication_failed", message: "secret" });
  expect(presentOperationError(error)).toContain("Authentication failed");
  expect(presentOperationError(error)).not.toContain("secret");
  expect(mapIpcError({ code: "unknown" }).code).toBe("request_failed");
});

test("mock adapters return decoded domain data through the same port", async () => {
  const { MockGrpcRepository, MockSettingsRepository } = await import("@/mocks/repositories");
  const grpc = new MockGrpcRepository();
  const operations = createFrontendOperations(grpc, grpc, new MockSettingsRepository());
  await operations.connect();
  expect((await operations.fetchGraphData(new Date(0), new Date())).size).toBe(2);
  expect((await operations.loadSettings()).url).toBe("https://mock.grpc.example");
});
