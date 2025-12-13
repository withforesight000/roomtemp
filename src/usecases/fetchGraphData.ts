import { GraphData } from "@/domain/types";
import { decodeIntoGraphData } from "@/frameworks/protobuf-ts";
import { GrpcRepository } from "@/interfaces/repositories/grpc";

export class FetchGraphData {
  constructor(private repo: GrpcRepository) {}

  async execute(startDate: Date, endDate: Date): Promise<GraphData> {
    const buffer = await this.repo.fetchGraphData(startDate, endDate);

    const data = await decodeIntoGraphData(buffer);
    return data;
  }
}
