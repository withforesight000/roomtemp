import type { FrontendOperations } from "@/usecases/frontendOperations";
export function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
export function operations(overrides: Partial<FrontendOperations> = {}): FrontendOperations {
  return {
    connect: async () => {}, fetchGraphData: async () => new Map(),
    loadSettings: async () => ({ url: "", accessToken: "", useProxies: false, proxyUrl: "" }),
    saveSettings: async () => {}, ...overrides,
  };
}
