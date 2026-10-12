import { expect, test } from "bun:test";
import { act, render, renderHook } from "@testing-library/react";
import { useGrpcConnect, type ConnectionSession } from "@/interfaces/react/hooks/useGrpcConnect";
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
test("a failed save preserves connection invalidation from an in-flight graph", async () => {
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
  expect(result.current.connection.state.isSaving).toBe(false);
  expect(result.current.connection.snapshot().ready).toBe(false);
  expect(result.current.connection.state.phase).toBe("error");
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

test("an old graph completion cannot release a newer pending graph's admission guard", async () => {
  const old = deferred<GraphData>();
  const newer = deferred<GraphData>();
  let calls = 0;
  const api = operations({ fetchGraphData: () => ++calls === 1 ? old.promise : newer.promise });
  const { result } = renderHook(() => {
    const connection = useGrpcConnect(api);
    return { connection, graph: useGraphData(api, connection) };
  });
  await flush();
  let oldRequest!: Promise<void>, newerRequest!: Promise<void>;
  act(() => { oldRequest = result.current.graph.fetch(...dates); });
  await act(async () => { await result.current.connection.connect(); });
  act(() => { newerRequest = result.current.graph.fetch(...dates); });
  await act(async () => { old.resolve(data); await oldRequest; });

  expect(result.current.graph.loading).toBe(true);
  expect(result.current.graph.data).toEqual([]);
  await act(async () => { await result.current.graph.fetch(...dates); });
  expect(calls).toBe(2);
  await act(async () => { newer.resolve(new Map()); await newerRequest; });
  expect(result.current.graph.loading).toBe(false);
  expect(result.current.graph.hasFetched).toBe(true);
  expect(result.current.graph.data).toEqual([]);
});

test("a failed save rejects an old render's fetch before the graph rerenders", async () => {
  let calls = 0;
  const api = operations({ fetchGraphData: async () => { calls++; return data; } });
  const { result } = renderHook(() => {
    const connection = useGrpcConnect(api);
    return { connection, graph: useGraphData(api, connection) };
  });
  await flush();
  await act(async () => { await result.current.graph.fetch(...dates); });
  expect(result.current.graph.data).toHaveLength(1);
  const oldFetch = result.current.graph.fetch;
  await act(async () => {
    const token = result.current.connection.beginSave()!;
    result.current.connection.finishSave(false, token);
    await oldFetch(...dates);
  });
  expect(calls).toBe(1);
  expect(result.current.connection.state.phase).toBe("ready");
  expect(result.current.graph.data).toEqual([]);
  expect(result.current.graph.hasFetched).toBe(false);
  await act(async () => { await result.current.graph.fetch(...dates); });
  expect(calls).toBe(2);
  expect(result.current.graph.data).toHaveLength(1);
});

for (const code of ["authentication_failed", "unavailable", "timeout", "request_failed"] as const) {
  test(`graph teardown forwards ${code} only when it invalidates the connection`, async () => {
    const response = deferred<GraphData>();
    const api = operations({ fetchGraphData: () => response.promise });
    let connection!: ConnectionSession;
    let graph!: ReturnType<typeof useGraphData>;
    function Graph({ session }: { session: ConnectionSession }) {
      graph = useGraphData(api, session);
      return null;
    }
    function Session({ showGraph }: { showGraph: boolean }) {
      connection = useGrpcConnect(api);
      return showGraph ? <Graph session={connection} /> : null;
    }
    const view = render(<Session showGraph />);
    await flush();
    let pending!: Promise<void>;
    act(() => { pending = graph.fetch(...dates); });
    view.rerender(<Session showGraph={false} />);
    await act(async () => { response.reject(new OperationFailure(code)); await pending; });
    expect(connection.state.phase).toBe(code === "request_failed" ? "ready" : "error");
    expect(connection.snapshot().ready).toBe(code === "request_failed");
  });
}
test("an obsolete graph failure cannot invalidate a newer connection", async () => {
  const response = deferred<GraphData>();
  const api = operations({ fetchGraphData: () => response.promise });
  const { result } = renderHook(() => {
    const connection = useGrpcConnect(api);
    return { connection, graph: useGraphData(api, connection) };
  });
  await flush();
  let pending!: Promise<void>;
  act(() => { pending = result.current.graph.fetch(...dates); });
  await act(async () => { await result.current.connection.connect(); });
  await act(async () => { response.reject(new OperationFailure("authentication_failed")); await pending; });
  expect(result.current.connection.state.phase).toBe("ready");
});

test("successful persistence reconnects after invalidation during saving", async () => {
  const api = operations();
  const { result } = renderHook(() => useGrpcConnect(api));
  await flush();
  const generation = result.current.snapshot().generation;
  let token!: NonNullable<ReturnType<ConnectionSession["beginSave"]>>;
  act(() => {
    token = result.current.beginSave()!;
    result.current.connectionFailed(new OperationFailure("timeout"), generation);
  });
  expect(result.current.state.phase).toBe("error");
  expect(result.current.state.isSaving).toBe(true);
  await act(async () => {
    result.current.finishSave(true, token);
    await result.current.connect();
    result.current.connectionFailed(new OperationFailure("authentication_failed"), generation);
  });
  expect(result.current.state.phase).toBe("ready");
  expect(result.current.state.isSaving).toBe(false);
});
