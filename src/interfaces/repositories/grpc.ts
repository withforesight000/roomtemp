import { invoke } from "@tauri-apps/api/core";
import {
  getMockGraphBuffer,
  mockGrpcOperation,
  recordMockConnection,
  isWebDriverMockEnabled,
} from "@/mocks/webdriver";

export interface GrpcRepository {
  connect(): Promise<string>;
  fetchGraphData(_startDate: Date, _endDate: Date): Promise<ArrayBuffer>;
}

export class GrpcRepositoryImpl implements GrpcRepository {
  async connect(): Promise<string> {
    if (isWebDriverMockEnabled()) {
      recordMockConnection();
      await mockGrpcOperation("connect");
      return "Mock connected to gRPC server";
    }
    return invoke<string>("connect_to_grpc_server");
  }

  async fetchGraphData(startDate: Date, endDate: Date): Promise<ArrayBuffer> {
    if (isWebDriverMockEnabled()) {
      const empty = await mockGrpcOperation("graph");
      return getMockGraphBuffer(empty);
    }
    return invoke<ArrayBuffer>("get_graph_data", {
      startTime: startDate.getTime(),
      endTime: endDate.getTime(),
    });
  }
}
