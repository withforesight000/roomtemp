import { invoke } from "@tauri-apps/api/core";
import { Settings } from "@/domain/types";
import {
  getMockSettings,
  failMockSettingsOperation,
  isWebDriverMockEnabled,
} from "@/mocks/webdriver";

export interface SettingsRepository {
  load(): Promise<Settings>;
  save(_settings: Settings): Promise<void>;
}

export class SettingsRepositoryImpl implements SettingsRepository {
  async load(): Promise<Settings> {
    if (isWebDriverMockEnabled()) {
      return getMockSettings();
    }
    return await invoke("get_settings");
  }

  async save(settings: Settings): Promise<void> {
    if (isWebDriverMockEnabled()) {
      failMockSettingsOperation("save");
      return;
    }
    await invoke("set_settings", settings);
  }
}
