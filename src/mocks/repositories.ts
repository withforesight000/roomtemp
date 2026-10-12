import type { ConnectionGateway, GraphDataRepository, SettingsRepository } from "@/usecases/ports/repositories";
import type { Settings } from "@/domain/types";
import { decodeIntoGraphData } from "@/frameworks/protobuf-ts";
import { grpcOperation } from "@/frameworks/ipcError";
import { getMockGraphBuffer, mockGrpcOperation, recordMockConnection, getMockSettings,
  waitMockSettingsSave, failMockSettingsOperation } from "./webdriver";
export class MockGrpcRepository implements ConnectionGateway, GraphDataRepository {
  connect(): Promise<void> {
    return grpcOperation(async () => { recordMockConnection(); await mockGrpcOperation("connect"); });
  }
  fetchGraphData() {
    return grpcOperation(async () => {
      const empty = await mockGrpcOperation("graph");
      return decodeIntoGraphData(await getMockGraphBuffer(empty));
    });
  }
}
export class MockSettingsRepository implements SettingsRepository {
  async load() { return getMockSettings(); }
  async save(_settings: Settings) { await waitMockSettingsSave(); failMockSettingsOperation("save"); }
}
