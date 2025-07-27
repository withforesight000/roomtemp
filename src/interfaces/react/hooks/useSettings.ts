import { useReducer } from "react";
import { SettingsRepository } from "@/interfaces/repositories/settings";
import { Settings } from "@/domain/types";

type State = Settings & { error?: string };

function reducer(state: State, action: { type: string; payload: Partial<State> }): State {
  switch (action.type) {
    case "LOAD":
      return { ...state, ...action.payload, error: undefined };
    case "SET_FIELD":
      console.log("Setting field:", action.payload);
      return { ...state, ...action.payload };
    case "SET_ERROR":
      return { ...state, error: action.payload.error };
    default:
      return state;
  }
}

export function useSettings(repo: SettingsRepository) {
  const [state, dispatch] = useReducer(reducer, { url: "", accessToken: "", error: undefined } as State);

  const load = async () => {
    try {
      const settings = await repo.load();
      dispatch({ type: "LOAD", payload: settings });
    } catch (error: any) {
      dispatch({ type: "SET_ERROR", payload: { error: error.message } });
    }
  };

  const save = async () => {
    try {
      await repo.save({ url: state.url, accessToken: state.accessToken });
      dispatch({ type: "SET_ERROR", payload: { error: undefined } });
    } catch (error: any) {
      dispatch({ type: "SET_ERROR", payload: { error: error.message } });
    }
  };

  return { state, dispatch, load, save };
}
