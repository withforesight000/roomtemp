import { expect, test } from "bun:test";
import { act, renderHook } from "@testing-library/react";
import { useGrpcConnect } from "@/interfaces/react/hooks/useGrpcConnect";
import { OperationFailure } from "@/usecases/operationError";
import { deferred, operations } from "./helpers";

test("a late connection failure cannot replace a newer success", async () => {
  const older = deferred<void>();
  let calls = 0;
  const api = operations({ connect: () => ++calls === 1 ? older.promise : Promise.resolve() });
  const { result } = renderHook(() => useGrpcConnect(api, "Connected to gRPC server"));
  let pending!: Promise<boolean>;
  act(() => { pending = result.current.connect(); });
  await act(async () => { await result.current.connect(); });
  await act(async () => { older.reject(new OperationFailure("authentication_failed")); await pending; });
  expect(result.current.state.phase).toBe("ready");
});
test("failed settings persistence preserves readiness without reconnecting", async () => {
  let calls = 0;
  const api = operations({ connect: async () => { calls++; } });
  const { result } = renderHook(() => useGrpcConnect(api, "Connected to gRPC server"));
  await act(async () => { await result.current.connect(); });
  const generation = result.current.state.generation;
  act(() => { result.current.beginSave(); result.current.finishSave(false); });
  expect(result.current.state.phase).toBe("ready");
  expect(result.current.state.generation).toBe(generation);
  expect(calls).toBe(1);
});
