import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "@/domain/types";
import type { SettingsRepository } from "@/usecases/ports/repositories";
export class TauriSettingsRepository implements SettingsRepository {
  load(): Promise<Settings> { return invoke("get_settings"); }
  save(settings: Settings): Promise<void> { return invoke("set_settings", settings); }
}
