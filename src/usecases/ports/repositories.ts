import type { GraphData, Settings } from "@/domain/types";

export interface ConnectionGateway { connect(): Promise<void>; }
export interface GraphDataRepository {
  fetchGraphData(startDate: Date, endDate: Date): Promise<GraphData>;
}
export interface SettingsRepository {
  load(): Promise<Settings>;
  save(settings: Settings): Promise<void>;
}
