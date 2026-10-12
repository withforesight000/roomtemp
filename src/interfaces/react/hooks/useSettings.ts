import { useCallback, useEffect, useReducer, useRef } from "react";
import type { Settings } from "@/domain/types";
import type { FrontendOperations } from "@/usecases/frontendOperations";
import { presentSettingsError } from "@/interfaces/presenters/errorPresenter";
import { initialSettingsState, settingsReducer, type SettingsFieldChange } from "../state/settings";
import type { SaveSettingsResult } from "./useSettingsWorkflow";

export function useSettings(
  operations: Pick<FrontendOperations, "loadSettings">,
  saveSettings: (values: Settings) => Promise<SaveSettingsResult>,
) {
  const [state, dispatch] = useReducer(settingsReducer, initialSettingsState);
  const mounted = useRef(true);
  const loadId = useRef(0);
  const saveId = useRef(0);
  const saveResultId = useRef(0);

  useEffect(() => {
    mounted.current = true;
    const id = ++loadId.current;
    const isCurrentLoad = () => mounted.current && id === loadId.current;

    async function load() {
      try {
        const values = await operations.loadSettings();
        if (isCurrentLoad()) dispatch({ type: "loaded", values });
      } catch (error) {
        if (isCurrentLoad()) {
          dispatch({ type: "error", message: presentSettingsError(error) });
        }
      }
    }

    void load();
    return () => {
      mounted.current = false;
      loadId.current++;
      saveResultId.current = ++saveId.current;
    };
  }, [operations]);

  const setField = useCallback((change: SettingsFieldChange) => {
    dispatch({ type: "fieldChanged", ...change });
  }, []);

  const save = useCallback(async () => {
    const id = ++saveId.current;
    // Discard loads started before this submission, including their errors.
    loadId.current++;
    const result = await saveSettings({ ...state.values });
    // An ignored duplicate must not suppress the accepted save's result.
    if (result.status === "ignored") return false;
    if (!mounted.current || id <= saveResultId.current) return result.status === "saved";

    saveResultId.current = id;
    dispatch({
      type: "error",
      message: result.status === "failed" ? presentSettingsError(result.error) : undefined,
    });
    return result.status === "saved";
  }, [saveSettings, state.values]);

  return { state, setField, save };
}
