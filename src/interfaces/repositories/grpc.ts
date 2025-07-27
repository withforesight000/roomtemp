import { invoke } from "@tauri-apps/api/core";

export interface GrpcRepository {
  connect(): Promise<string>;
  fetchGraphData(startDate: Date, endDate: Date): Promise<ArrayBuffer>;
}

export class GrpcRepositoryImpl implements GrpcRepository {
  async connect(): Promise<string> {
    return invoke<string>("connect_to_grpc_server");
  }

  async fetchGraphData(startDate: Date, endDate: Date): Promise<ArrayBuffer> {
    return invoke<ArrayBuffer>("get_graph_data", {
      startTime: startDate.getTime(),
      endTime: endDate.getTime(),
    });
  }
}
