import { expect, test } from "bun:test";
import { act, renderHook } from "@testing-library/react";
import { useGrpcConnect } from "@/interfaces/react/hooks/useGrpcConnect";
import { useGraphData } from "@/interfaces/react/hooks/useGraphData";
import { connectionReducer, initialConnectionState } from "@/interfaces/react/state/connection";
import { graphReducer, initialGraphState } from "@/interfaces/react/state/graph";
import type { GraphData } from "@/domain/types";
import { OperationFailure } from "@/usecases/operationError";
import { deferred, operations } from "./helpers";
const data: GraphData = new Map([["1000-0", { temperature: 22, humidity: 48, illumination: 120 }]]);
const dates = [new Date(0), new Date(2000)] as const;
async function flush() { await act(async () => {}); }

test("reducers reject obsolete completions and preserve saving across connection completion", () => {
  let state = connectionReducer(initialConnectionState, { type: "checking", generation: 1, graphEpoch: 1 });
  state = connectionReducer(state, { type: "saveStarted", graphEpoch: 2 });
  expect(connectionReducer(state, { type: "ready", generation: 0 })).toBe(state);
  expect(connectionReducer(state, { type: "ready", generation: 1 }).isSaving).toBe(true);
  const graph = graphReducer(initialGraphState, { type: "started", generation: 2, graphEpoch: 2, request: 4 });
  expect(graphReducer(graph, { type: "succeeded", generation: 1, graphEpoch: 2, request: 4, data: [] })).toBe(graph);
});
test("a late success cannot replace a newer connection error", async () => {
  const older = deferred<void>();
  let calls = 0;
  const api = operations({ connect: () => ++calls === 2 ? older.promise : calls === 3 ? Promise.reject(new OperationFailure("unavailable")) : Promise.resolve() });
  const { result } = renderHook(() => useGrpcConnect(api));
  await flush();
  let pending!: Promise<boolean>;
  act(() => { pending = result.current.connect(); });
  await act(async () => { await result.current.connect(); });
  await act(async () => { older.resolve(); await pending; });
  expect(result.current.state.phase).toBe("error");
  expect(result.current.snapshot().ready).toBe(false);
});
test("Strict Mode cleanup discards the first setup's connection", async () => {
  const first = deferred<void>();
  let calls = 0;
  const api = operations({ connect: () => ++calls === 1 ? first.promise : Promise.resolve() });
  const { result } = renderHook(() => useGrpcConnect(api), {
    reactStrictMode: true,
  });
  await flush();
  expect(calls).toBe(2);
  await act(async () => { first.reject(new OperationFailure("authentication_failed")); });
  expect(result.current.state.phase).toBe("ready");
});
test("provider teardown suppresses late connection completion", async () => {
  const pending = deferred<void>();
  const api = operations({ connect: () => pending.promise });
  const { result, unmount } = renderHook(() => useGrpcConnect(api));
  const snapshot = result.current.snapshot;
  unmount();
  await act(async () => { pending.resolve(); });
  expect(snapshot().active).toBe(false);
  expect(snapshot().ready).toBe(false);
});
test("graph admission is synchronous and manual", async () => {
  const response = deferred<GraphData>();
  let calls = 0;
  const api = operations({ fetchGraphData: () => { calls++; return response.promise; } });
  const { result } = renderHook(() => {
    const connection = useGrpcConnect(api);
    return { connection, graph: useGraphData(api, connection) };
  });
  await flush();
  expect(calls).toBe(0);
  let pending!: Promise<void>;
  act(() => { pending = result.current.graph.fetch(...dates); void result.current.graph.fetch(...dates); });
  expect(calls).toBe(1);
  await act(async () => { response.resolve(data); await pending; });
  expect(result.current.graph.data).toHaveLength(1);
});
test("save admission blocks fetch and stale failure before the next render", async () => {
  const response = deferred<GraphData>();
  let calls = 0;
  const api = operations({ fetchGraphData: () => { calls++; return response.promise; } });
  const { result } = renderHook(() => {
    const connection = useGrpcConnect(api);
    return { connection, graph: useGraphData(api, connection) };
  });
  await flush();
  let pending!: Promise<void>;
  act(() => { pending = result.current.graph.fetch(...dates); });
  await act(async () => {
    const token = result.current.connection.beginSave()!;
    await result.current.graph.fetch(...dates);
    response.reject(new OperationFailure("authentication_failed"));
    await pending;
    result.current.connection.finishSave(false, token);
  });
  expect(calls).toBe(1);
  expect(result.current.connection.state.phase).toBe("ready");
  expect(result.current.graph.data).toEqual([]);
  expect(result.current.graph.hasFetched).toBe(false);
});
test("quiet cancellation releases loading and unavailable invalidates the connection", async () => {
  let code: "superseded" | "unavailable" = "superseded";
  const api = operations({ fetchGraphData: async () => { throw new OperationFailure(code); } });
  const { result } = renderHook(() => {
    const connection = useGrpcConnect(api);
    return { connection, graph: useGraphData(api, connection) };
  });
  await flush();
  await act(async () => { await result.current.graph.fetch(...dates); });
  expect(result.current.graph.loading).toBe(false);
  expect(result.current.graph.error).toBeNull();
  expect(result.current.graph.hasFetched).toBe(false);
  code = "unavailable";
  await act(async () => { await result.current.graph.fetch(...dates); });
  expect(result.current.connection.state.phase).toBe("error");
  expect(result.current.graph.ready).toBe(false);
});
test("an old graph cannot replace a newer result after reconnect", async () => {
  const old = deferred<GraphData>();
  let calls = 0;
  const api = operations({ fetchGraphData: () => ++calls === 1 ? old.promise : Promise.resolve(new Map()) });
  const { result } = renderHook(() => {
    const connection = useGrpcConnect(api);
    return { connection, graph: useGraphData(api, connection) };
  });
  await flush();
  let pending!: Promise<void>;
  act(() => { pending = result.current.graph.fetch(...dates); });
  await act(async () => { await result.current.connection.connect(); });
  await act(async () => { await result.current.graph.fetch(...dates); });
  await act(async () => { old.resolve(data); await pending; });
  expect(result.current.graph.data).toEqual([]);
  expect(result.current.graph.hasFetched).toBe(true);
});
