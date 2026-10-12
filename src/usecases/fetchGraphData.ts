import type { GraphDataRepository } from "./ports/repositories";
export class FetchGraphData {
  constructor(private readonly repo: GraphDataRepository) {}
  execute(startDate: Date, endDate: Date) { return this.repo.fetchGraphData(startDate, endDate); }
}
