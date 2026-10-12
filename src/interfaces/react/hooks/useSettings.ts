import { useCallback, useEffect, useReducer, useRef } from "react";
import type { Settings } from "@/domain/types";
import type { FrontendOperations } from "@/usecases/frontendOperations";
import { presentSettingsError } from "@/interfaces/presenters/errorPresenter";
import { initialSettingsState, settingsReducer, type SettingsFieldChange } from "../state/settings";
import type { SaveSettingsResult } from "./useSettingsWorkflow";

export function useSettings(operations: Pick<FrontendOperations, "loadSettings">,
  saveSettings: (values: Settings) => Promise<SaveSettingsResult>) {
  const [state, dispatch] = useReducer(settingsReducer, initialSettingsState);
  const mounted = useRef(true);
  const loadId = useRef(0);
  const saveId = useRef(0);
  const saveResultId = useRef(0);
  const setField = useCallback((change: SettingsFieldChange) => {
    dispatch({ type: "fieldChanged", ...change });
  }, []);
  const load = useCallback(async () => {
    const id = ++loadId.current;
    try {
      const values = await operations.loadSettings();
      if (mounted.current && id === loadId.current) dispatch({ type: "loaded", values });
    } catch (error) {
      if (mounted.current && id === loadId.current) dispatch({ type: "error", message: presentSettingsError(error) });
    }
  }, [operations]);
  const save = useCallback(async () => {
    const id = ++saveId.current;
    loadId.current++; // Loading an older persisted snapshot must not replace a submitted form.
    const result = await saveSettings({ ...state.values });
    if (mounted.current && id > saveResultId.current && result.status !== "ignored") {
      saveResultId.current = id;
      dispatch({ type: "error", message: result.status === "failed" ? presentSettingsError(result.error) : undefined });
    }
    return result.status === "saved";
  }, [saveSettings, state.values]);
  useEffect(() => {
    mounted.current = true;
    void load();
    return () => { mounted.current = false; loadId.current++; saveResultId.current = ++saveId.current; };
  }, [load]);
  return { state, setField, save };
}
