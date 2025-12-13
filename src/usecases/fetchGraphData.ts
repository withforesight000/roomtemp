import { GraphData } from "@/domain/types";
import { decodeIntoGraphData } from "@/frameworks/protobuf-ts";
import { GrpcRepository } from "@/interfaces/repositories/grpc";

export class FetchGraphData {
  private repo: GrpcRepository;
  constructor(repo: GrpcRepository) {
    this.repo = repo;
  }

  async execute(startDate: Date, endDate: Date): Promise<GraphData> {
    const buffer = await this.repo.fetchGraphData(startDate, endDate);

    const data = await decodeIntoGraphData(buffer);
    return data;
  }
}
