import { GraphData } from "@/domain/types";
import { decodeIntoGraphData } from "@/frameworks/protobuf-ts";
import { Grpc } from "@/interfaces/repositories/grpc";

export class FetchGraphData {
  constructor(private repo: Grpc) {}

  async execute(startDate: Date, endDate: Date): Promise<GraphData> {
    const buffer = await this.repo.fetchGraphData(startDate, endDate);

    const data = decodeIntoGraphData(buffer);
    return data;
  }
}
