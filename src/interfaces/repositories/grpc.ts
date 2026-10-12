import { invoke } from "@tauri-apps/api/core";
import type { ConnectionGateway, GraphDataRepository } from "@/usecases/ports/repositories";
import { decodeIntoGraphData } from "@/frameworks/protobuf-ts";
import { grpcOperation } from "@/frameworks/ipcError";

export class TauriGrpcRepository implements ConnectionGateway, GraphDataRepository {
  connect(): Promise<void> {
    return grpcOperation(async () => { await invoke("connect_to_grpc_server"); });
  }
  fetchGraphData(startDate: Date, endDate: Date) {
    return grpcOperation(async () => {
      const buffer = await invoke<ArrayBuffer>("get_graph_data", {
        startTime: startDate.getTime(), endTime: endDate.getTime(),
      });
      return decodeIntoGraphData(buffer);
    });
  }
}
