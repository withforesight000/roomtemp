export type AmbientCondition = {
  temperature: number;
  humidity: number;
  illumination: number;
};

export type GraphData = Map<string, AmbientCondition>;
