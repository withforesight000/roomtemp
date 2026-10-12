import { expect, test } from "bun:test";
import { act, render, renderHook } from "@testing-library/react";
import { useSettings } from "@/interfaces/react/hooks/useSettings";
import { useSettingsWorkflow } from "@/interfaces/react/hooks/useSettingsWorkflow";
import { useGrpcConnect } from "@/interfaces/react/hooks/useGrpcConnect";
import { GrpcRepoProvider, useFrontend, type GrpcContextValue } from "@/interfaces/react/contexts/grpcRepoContext";
import type { Settings } from "@/domain/types";
import { OperationFailure } from "@/usecases/operationError";
import { deferred, operations } from "./helpers";
const stored: Settings = { url: "https://stored.example", accessToken: "disposable", useProxies: true, proxyUrl: "http://proxy.example" };
async function flush() { await act(async () => {}); }
function harness(api: ReturnType<typeof operations>) {
  return renderHook(() => {
    const connection = useGrpcConnect(api);
    const saveSettings = useSettingsWorkflow(api, connection);
    const form = useSettings(api, saveSettings);
    return { connection, saveSettings, form };
  });
}

test("a delayed load fills untouched fields while preserving edits", async () => {
  const load = deferred<Settings>();
  const api = operations({ loadSettings: () => load.promise });
  const { result } = harness(api);
  act(() => { result.current.form.setField({ field: "url", value: "https://edited.example" }); });
  await act(async () => { load.resolve(stored); });
  expect(result.current.form.state.values).toEqual({ ...stored, url: "https://edited.example" });
});
test("Strict Mode's earlier settings load cannot replace the latest load", async () => {
  const old = deferred<Settings>();
  let calls = 0;
  const api = operations({ loadSettings: () => ++calls === 1 ? old.promise : Promise.resolve(stored) });
  const save = async () => ({ status: "saved", connected: true } as const);
  const { result } = renderHook(() => useSettings(api, save), { reactStrictMode: true });
  await flush();
  expect(calls).toBe(2);
  await act(async () => { old.resolve({ ...stored, url: "https://obsolete.example" }); });
  expect(result.current.state.values).toEqual(stored);
});
test("a load begun before a save cannot replace submitted fields", async () => {
  const load = deferred<Settings>();
  const api = operations({ loadSettings: () => load.promise });
  const { result } = harness(api);
  await flush();
  act(() => { result.current.form.setField({ field: "url", value: "https://new.example" }); });
  await act(async () => { await result.current.form.save(); });
  await act(async () => { load.resolve(stored); });
  expect(result.current.form.state.values.url).toBe("https://new.example");
  expect(result.current.form.state.values.accessToken).toBe("");
});
test("duplicate save admission persists once and does not hide the actual failure", async () => {
  const persistence = deferred<void>();
  let calls = 0;
  const api = operations({ saveSettings: () => { calls++; return persistence.promise; } });
  const { result } = harness(api);
  await flush();
  let first!: Promise<boolean>, duplicate!: Promise<boolean>;
  act(() => { first = result.current.form.save(); duplicate = result.current.form.save(); });
  expect(calls).toBe(1);
  await act(async () => { persistence.reject("Keystore: storage unavailable"); await Promise.all([first, duplicate]); });
  expect(result.current.form.state.error).toBe("Keystore: storage unavailable");
  expect(result.current.connection.state.phase).toBe("ready");
});
test("save uses a fixed snapshot and retains it when health verification fails", async () => {
  const saved: Settings[] = [];
  let calls = 0;
  const api = operations({ saveSettings: async values => { saved.push(values); },
    connect: async () => { if (++calls === 2) throw new OperationFailure("authentication_failed"); } });
  const { result } = harness(api);
  await flush();
  const input = { ...stored };
  let pending!: Promise<unknown>;
  act(() => { pending = result.current.saveSettings(input); input.url = "changed after submission"; });
  await act(async () => { expect(await pending).toEqual({ status: "saved", connected: false }); });
  expect(saved).toEqual([stored]);
  expect(result.current.connection.state.phase).toBe("error");
});
test("re-save supersedes an older reconnect and its late failure", async () => {
  const oldConnection = deferred<void>();
  let calls = 0;
  const api = operations({ connect: () => ++calls === 2 ? oldConnection.promise : Promise.resolve() });
  const { result } = harness(api);
  await flush();
  let oldSave!: Promise<unknown>;
  await act(async () => { oldSave = result.current.saveSettings(stored); });
  expect(result.current.connection.state.phase).toBe("checking");
  await act(async () => { await result.current.saveSettings({ ...stored, url: "https://new.example" }); });
  await act(async () => { oldConnection.reject(new OperationFailure("authentication_failed")); await oldSave; });
  expect(result.current.connection.state.phase).toBe("ready");
  expect(result.current.connection.state.isSaving).toBe(false);
});
test("connection completion during a later failed save remains valid and preserves the newer form error", async () => {
  const connection = deferred<void>();
  const persistence = deferred<void>();
  let connections = 0, saves = 0;
  const api = operations({ connect: () => ++connections === 2 ? connection.promise : Promise.resolve(),
    saveSettings: () => ++saves === 2 ? persistence.promise : Promise.resolve() });
  const { result } = harness(api);
  await flush();
  let first!: Promise<boolean>, second!: Promise<boolean>;
  await act(async () => { first = result.current.form.save(); });
  act(() => { second = result.current.form.save(); });
  await act(async () => { connection.resolve(); await first; });
  expect(result.current.connection.state.phase).toBe("ready");
  expect(result.current.connection.state.isSaving).toBe(true);
  await act(async () => { persistence.reject("Keystore: storage unavailable"); await second; });
  expect(result.current.connection.state.phase).toBe("ready");
  expect(result.current.form.state.error).toBe("Keystore: storage unavailable");
  expect(connections).toBe(2);
});
test("an old form save result cannot erase the newer failure", async () => {
  const connection = deferred<void>();
  let connects = 0, saves = 0;
  const api = operations({ connect: () => ++connects === 2 ? connection.promise : Promise.resolve(),
    saveSettings: async () => { if (++saves === 2) throw "newer save failed"; } });
  const { result } = harness(api);
  await flush();
  let first!: Promise<boolean>;
  await act(async () => { first = result.current.form.save(); });
  await act(async () => { await result.current.form.save(); });
  await act(async () => { connection.resolve(); await first; });
  expect(result.current.form.state.error).toBe("newer save failed");
});
test("successful persistence completes reconnection after the form unmounts", async () => {
  const persistence = deferred<void>();
  let connects = 0;
  const api = operations({ saveSettings: () => persistence.promise, connect: async () => { connects++; } });
  const services = { operations: api, now: () => 0, connectedMessage: "Connected" };
  let frontend!: GrpcContextValue;
  let form!: ReturnType<typeof useSettings>;
  function Observer() { frontend = useFrontend(); return null; }
  function Form() { const api = useFrontend(); form = useSettings(api.operations, api.saveSettings); return null; }
  const view = render(<GrpcRepoProvider services={services}><Observer /><Form /></GrpcRepoProvider>);
  await flush();
  let pending!: Promise<boolean>;
  act(() => { pending = form.save(); });
  view.rerender(<GrpcRepoProvider services={services}><Observer /></GrpcRepoProvider>);
  await act(async () => { persistence.resolve(); await pending; });
  expect(connects).toBe(2);
  expect(frontend.state.phase).toBe("ready");
});
test("provider teardown lets storage finish without reconnecting the disposed session", async () => {
  const persistence = deferred<void>();
  let connects = 0;
  const api = operations({ saveSettings: () => persistence.promise, connect: async () => { connects++; } });
  const { result, unmount } = harness(api);
  await flush();
  let pending!: Promise<unknown>;
  act(() => { pending = result.current.saveSettings(stored); });
  unmount();
  await act(async () => { persistence.resolve(); expect(await pending).toEqual({ status: "saved", connected: false }); });
  expect(connects).toBe(1);
});
