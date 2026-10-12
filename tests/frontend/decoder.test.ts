import { expect, test } from "bun:test";
import { GetAmbientConditionsResponse } from "@withforesight/tempgrpcd-protos";
import { decodeIntoGraphData } from "@/frameworks/protobuf-ts";
import { invalidatesConnection, OperationFailure } from "@/usecases/operationError";
const value = { temperature: 22.5, humidity: 48, illumination: 120 };
const payload = { "1790856000000-0": value };
const json = (value: unknown) => Uint8Array.from(new TextEncoder().encode(JSON.stringify(value))).buffer;
const protobuf = (ambientConditions: Record<string, typeof value>) =>
  Uint8Array.from(GetAmbientConditionsResponse.toBinary({ ambientConditions })).buffer;
test("valid protobuf and legacy JSON preserve keys and measurements", async () => {
  const expected = new Map(Object.entries(payload));
  expect(await decodeIntoGraphData(protobuf(payload))).toEqual(expected);
  expect(await decodeIntoGraphData(json(payload))).toEqual(expected);
});
test("valid empty protobuf and JSON remain successful empty results", async () => {
  for (const buffer of [protobuf({}), json({})]) expect(await decodeIntoGraphData(buffer)).toEqual(new Map());
});
test("malformed bytes, shapes, timestamps and measurements fail without invalidating the connection", async () => {
  const cases = [new Uint8Array([255]).buffer, json(null), json([]), json({ "bad-0": value }),
    json({ "1790856000000-0": { ...value, humidity: "48" } }),
    protobuf({ "1790856000000-0": { ...value, temperature: Number.NaN } }),
    protobuf({ "99999999999999999999-0": value })];
  for (const buffer of cases) {
    let failure: unknown;
    try { await decodeIntoGraphData(buffer); } catch (error) { failure = error; }
    expect(failure).toBeInstanceOf(OperationFailure);
    expect((failure as OperationFailure).code).toBe("request_failed");
    expect(invalidatesConnection(failure as OperationFailure)).toBe(false);
  }
});
