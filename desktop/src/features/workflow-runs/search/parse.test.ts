import { expect, test } from "vitest";
import { getFilterValue, toFilter, toTokenSegment } from "./parse";
import { isOk } from "@/lib/result";

test("parses search string into filter", () => {
  expect(toFilter("@workflow:run-123")).toEqual({
    success: true,
    data: { entity: "workflow", id: "run-123" },
  });

  expect(toFilter("@tag:name")).toEqual({
    success: true,
    data: { entity: "tag", name: "name", value: undefined },
  });

  expect(toFilter("@tag:name:value")).toEqual({
    success: true,
    data: { entity: "tag", name: "name", value: "value" },
  });

  expect(isOk(toFilter("@workflow:"))).toEqual(false);
  expect(isOk(toFilter("@tag:"))).toEqual(false);
});

test("converts filter into token segment", () => {
  expect(toTokenSegment({ entity: "workflow", id: "run-123" })).toEqual({
    type: "token",
    text: "@workflow:run-123",
    value: { entity: "workflow", id: "run-123" },
  });

  expect(toTokenSegment({ entity: "tag", name: "name", value: undefined })).toEqual({
    type: "token",
    text: "@tag:name",
    value: { entity: "tag", name: "name", value: undefined },
  });

  expect(toTokenSegment({ entity: "tag", name: "name", value: "value" })).toEqual({
    type: "token",
    text: "@tag:name:value",
    value: { entity: "tag", name: "name", value: "value" },
  });
});

test("extract value of the filter without a prefix", () => {
  expect(getFilterValue("@workf")).toEqual("@workf");
  expect(getFilterValue("@tag")).toEqual("@tag");
  expect(getFilterValue("@tag:name")).toEqual("name");
  expect(getFilterValue("@workflow:something")).toEqual("something");
});
