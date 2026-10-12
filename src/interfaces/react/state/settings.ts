import type { Settings } from "@/domain/types";
export type SettingsFieldChange = {
  [Field in keyof Settings]: { field: Field; value: Settings[Field] }
}[keyof Settings];
export type SettingsState = {
  values: Settings;
  edited: Partial<Record<keyof Settings, true>>;
  error?: string;
};
export const initialSettingsState: SettingsState = {
  values: { url: "", accessToken: "", useProxies: false, proxyUrl: "" }, edited: {},
};
export type SettingsAction =
  | ({ type: "fieldChanged" } & SettingsFieldChange)
  | { type: "loaded"; values: Settings }
  | { type: "error"; message?: string };
export function settingsReducer(state: SettingsState, action: SettingsAction): SettingsState {
  switch (action.type) {
    case "fieldChanged": return { ...state, values: { ...state.values, [action.field]: action.value }, edited: { ...state.edited, [action.field]: true } };
    case "loaded": {
      const values = { ...action.values };
      // A late load fills untouched fields but never overwrites the user's edits.
      if (state.edited.url) values.url = state.values.url;
      if (state.edited.accessToken) values.accessToken = state.values.accessToken;
      if (state.edited.useProxies) values.useProxies = state.values.useProxies;
      if (state.edited.proxyUrl) values.proxyUrl = state.values.proxyUrl;
      return { ...state, values, error: undefined };
    }
    case "error": return { ...state, error: action.message };
  }
}
