import { expect, test } from "vitest";
import { toFilter } from "./parse";
import { isOk } from "@/lib/result";

test("parses a workflow filter", () => {
  expect(toFilter("@workflow:run-123")).toEqual({
    success: true,
    data: { entity: "workflow", id: "run-123" },
  });

  expect(isOk(toFilter("@workflow:"))).toEqual(false);
});
