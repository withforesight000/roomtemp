export type AmbientCondition = {
  temperature: number;
  humidity: number;
  illumination: number;
};

export type GraphData = Map<string, AmbientCondition>;

export type Settings = {
  url: string;
  accessToken: string;
  useProxies: boolean;
  proxyUrl: string;
};
