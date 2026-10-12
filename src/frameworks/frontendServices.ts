import { createFrontendOperations, type FrontendOperations } from "@/usecases/frontendOperations";
import { TauriGrpcRepository } from "@/interfaces/repositories/grpc";
import { TauriSettingsRepository } from "@/interfaces/repositories/settings";
import { MockGrpcRepository, MockSettingsRepository } from "@/mocks/repositories";
import { isWebDriverMockEnabled, MOCK_REFERENCE_TIME } from "@/mocks/webdriver";

export interface FrontendServices {
  operations: FrontendOperations;
  now: () => number;
  connectedMessage: string;
}
/** The only place choosing production or WebDriver dependencies. */
export function createFrontendServices(): FrontendServices {
  const mock = isWebDriverMockEnabled();
  const grpc = mock ? new MockGrpcRepository() : new TauriGrpcRepository();
  const settings = mock ? new MockSettingsRepository() : new TauriSettingsRepository();
  return {
    operations: createFrontendOperations(grpc, grpc, settings),
    now: mock ? () => MOCK_REFERENCE_TIME : Date.now,
    connectedMessage: mock ? "Mock connected to gRPC server" : "Connected to gRPC server",
  };
}
