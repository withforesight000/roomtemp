import { GraphData } from "@/domain/types";
import { decoder, GraphDataRustType } from "@/frameworks/bincode";
import { Grpc } from "@/interfaces/repositories/grpc";

export class FetchGraphData {
  constructor(private repo: Grpc) {}

  async execute(startDate: Date, endDate: Date): Promise<GraphData> {
    const buffer = await this.repo.fetchGraphData(startDate, endDate);
    const bytes = new Uint8Array(buffer);
    const data = decoder.load(bytes).decodeAs<GraphData>(GraphDataRustType);
    return data;
  }
}
