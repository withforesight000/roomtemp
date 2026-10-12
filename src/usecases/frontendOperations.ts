import type { GraphData, Settings } from "@/domain/types";
import type { ConnectionGateway, GraphDataRepository, SettingsRepository } from "./ports/repositories";
import { FetchGraphData } from "./fetchGraphData";

export interface FrontendOperations {
  connect(): Promise<void>;
  fetchGraphData(start: Date, end: Date): Promise<GraphData>;
  loadSettings(): Promise<Settings>;
  saveSettings(settings: Settings): Promise<void>;
}
export function createFrontendOperations(connection: ConnectionGateway, graphs: GraphDataRepository,
  settings: SettingsRepository): FrontendOperations {
  const fetchGraphData = new FetchGraphData(graphs);
  return {
    connect: () => connection.connect(),
    fetchGraphData: (start, end) => fetchGraphData.execute(start, end),
    loadSettings: () => settings.load(),
    saveSettings: values => settings.save(values),
  };
}
