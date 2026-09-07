import { expect, test } from "vitest";
import { getFilterValue, toFilter, toTokenSegment } from "./parse";
import { isOk } from "@/lib/result";
import { searchParamsSchema } from "./search-params";

test("parses search string into filter", () => {
  expect(toFilter("@workflow:run-123")).toEqual({
    success: true,
    data: { entity: "workflow", id: "run-123" },
  });

  expect(toFilter("@tag:production")).toEqual({
    success: true,
    data: { entity: "tag", value: "production" },
  });

  expect(toFilter("@tag:region:us")).toEqual({
    success: true,
    data: { entity: "tag", value: "region:us" },
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
  expect(toTokenSegment({ entity: "tag", value: "production" })).toEqual({
    type: "token",
    text: "@tag:production",
    value: { entity: "tag", value: "production" },
  });

  expect(toTokenSegment({ entity: "tag", value: "region:us" })).toEqual({
    type: "token",
    text: "@tag:region:us",
    value: { entity: "tag", value: "region:us" },
  });
});

test.each(["production", "region:us:east", "label:"])(
  "preserves the complete tag value %s",
  (value) => {
    const text = `@tag:${value}`;
    const result = toFilter(text);

    expect(result).toEqual({ success: true, data: { entity: "tag", value } });
    if (!isOk(result)) throw new Error("Expected a valid tag filter");

    expect(toTokenSegment(result.data).text).toBe(text);
    expect(getFilterValue(text)).toBe(value);
  },
);

test("validates tag search params as a single required string", () => {
  const filters = [{ entity: "tag", value: "region:us:east" }];
  expect(searchParamsSchema.parse({ filters })).toEqual({ filters });
  expect(searchParamsSchema.parse({ filters: [{ entity: "tag" }] })).toEqual({
    filters: undefined,
  });
  expect(searchParamsSchema.parse({ filters: [{ entity: "tag", value: "" }] })).toEqual({
    filters: undefined,
  });
});

test("extract value of the filter without a prefix", () => {
  expect(getFilterValue("@workf")).toEqual("@workf");
  expect(getFilterValue("@tag")).toEqual("@tag");
  expect(getFilterValue("@tag:production")).toEqual("production");
  expect(getFilterValue("@workflow:something")).toEqual("something");
});
