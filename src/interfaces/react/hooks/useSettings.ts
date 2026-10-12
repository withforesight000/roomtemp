import { useCallback, useReducer } from "react";
import { SettingsRepository } from "@/interfaces/repositories/settings";
import { Settings } from "@/domain/types";

type State = Settings & { error?: string };

function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Settings could not be accessed";
}

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

export function useSettings(repo: SettingsRepository) {
  const [state, dispatch] = useReducer(reducer, {
    url: "",
    accessToken: "",
    useProxies: false,
    proxyUrl: "",
    error: undefined,
  } as State);

  const load = useCallback(async () => {
    try {
      const settings = await repo.load();
      dispatch({ type: "LOAD", payload: settings });
    } catch (error: unknown) {
      dispatch({ type: "SET_ERROR", payload: { error: errorMessage(error) } });
    }
  }, [repo]);

  const save = useCallback(async () => {
    try {
      await repo.save({
        url: state.url,
        accessToken: state.accessToken,
        useProxies: state.useProxies,
        proxyUrl: state.proxyUrl,
      });
      dispatch({ type: "SET_ERROR", payload: { error: undefined } });
      return true;
    } catch (error: unknown) {
      dispatch({ type: "SET_ERROR", payload: { error: errorMessage(error) } });
    }
    return false;
  }, [repo, state.accessToken, state.proxyUrl, state.url, state.useProxies]);

  return { state, dispatch, load, save };
}
