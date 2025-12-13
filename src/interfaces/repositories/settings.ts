import { invoke } from "@tauri-apps/api/core";
import { Settings } from "@/domain/types";

export interface SettingsRepository {
  load(): Promise<Settings>;
  save(_settings: Settings): Promise<void>;
}

export class SettingsRepositoryImpl implements SettingsRepository {
  async load(): Promise<Settings> {
    return await invoke("get_settings");
  }

  async save(settings: Settings): Promise<void> {
    await invoke("set_settings", settings);
  }
}
