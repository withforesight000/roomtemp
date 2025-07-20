import { invoke } from "@tauri-apps/api/core";

export interface Grpc {
  connect(): Promise<string>;
  fetchGraphData(startDate: Date, endDate: Date): Promise<ArrayBuffer>;
}

export class GrpcImpl implements Grpc {
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
