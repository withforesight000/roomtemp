import { useCallback } from "react";
import type { Settings } from "@/domain/types";
import type { FrontendOperations } from "@/usecases/frontendOperations";
import type { ConnectionSession } from "./useGrpcConnect";

export type SaveSettingsResult =
  | { status: "saved"; connected: boolean }
  | { status: "failed"; error: unknown }
  | { status: "ignored" };

/** Provider lifetime: persisting settings must outlive the form that submitted them. */
export function useSettingsWorkflow(
  operations: Pick<FrontendOperations, "saveSettings">,
  connection: ConnectionSession,
) {
  const { beginSave, finishSave, connect } = connection;
  return useCallback(async (values: Settings): Promise<SaveSettingsResult> => {
    const token = beginSave();
    if (!token) return { status: "ignored" };

    const settings = { ...values };
    try {
      await operations.saveSettings(settings);
    } catch (error) {
      finishSave(false, token);
      return { status: "failed", error };
    }
    // A page unmount does not cancel this step; provider teardown prevents callbacks instead.
    const active = finishSave(true, token);
    const connected = active ? await connect() : false;
    return { status: "saved", connected };
  }, [operations, beginSave, finishSave, connect]);
}
