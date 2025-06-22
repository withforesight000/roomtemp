import { GraphData } from "@/domain/types";

export type RechartPoint = {
  time: string;
  temperature: number;
  humidity: number;
  illumination: number;
};

export function presentGraphData(graphData: GraphData): RechartPoint[] {
  return Array.from(graphData.entries())
    .map(([key, v]) => {
      const ms = Number(key.split('-')[0]);
      return {
        time: new Date(ms).toLocaleString(),
        temperature: v.temperature,
        humidity: v.humidity,
        illumination: v.illumination,
      };
    })
    .sort((a, b) => a.time.localeCompare(b.time));
}
