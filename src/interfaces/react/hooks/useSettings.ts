import { useCallback, useReducer } from "react";
import type { FrontendOperations } from "@/usecases/frontendOperations";
import { presentSettingsError } from "@/interfaces/presenters/errorPresenter";
import { Settings } from "@/domain/types";

type State = Settings & { error?: string };

function reducer(
  state: State,
  action: { type: string; payload: Partial<State> }
): State {
  switch (action.type) {
    case "LOAD":
      return { ...state, ...action.payload, error: undefined };
    case "SET_FIELD":
      return { ...state, ...action.payload };
    case "SET_ERROR":
      return { ...state, error: action.payload.error };
    default:
      return state;
  }
}

export function useSettings(operations: FrontendOperations) {
  const [state, dispatch] = useReducer(reducer, {
    url: "",
    accessToken: "",
    useProxies: false,
    proxyUrl: "",
    error: undefined,
  } as State);

  const load = useCallback(async () => {
    try {
      const settings = await operations.loadSettings();
      dispatch({ type: "LOAD", payload: settings });
    } catch (error: unknown) {
      dispatch({ type: "SET_ERROR", payload: { error: presentSettingsError(error) } });
    }
  }, [operations]);

  const save = useCallback(async () => {
    try {
      await operations.saveSettings({
        url: state.url,
        accessToken: state.accessToken,
        useProxies: state.useProxies,
        proxyUrl: state.proxyUrl,
      });
      dispatch({ type: "SET_ERROR", payload: { error: undefined } });
      return true;
    } catch (error: unknown) {
      dispatch({ type: "SET_ERROR", payload: { error: presentSettingsError(error) } });
    }
    return false;
  }, [operations, state.accessToken, state.proxyUrl, state.url, state.useProxies]);

  return { state, dispatch, load, save };
}
