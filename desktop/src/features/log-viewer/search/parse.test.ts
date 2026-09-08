import { expect, test } from "vitest";

import { getFilterValue, toTokenSegment } from "./parse";

test("token chips show the short public id and keep the numeric query value", () => {
  expect(
    toTokenSegment({
      entity: "job_run",
      job_run_id: 42,
      public_id: "0193e8c2-7b1a-7c3d-9f2e-1a2b3c4d5e6f",
    }),
  ).toEqual({
    type: "token",
    text: "@job_run:5e6f",
    value: {
      entity: "job_run",
      job_run_id: 42,
      public_id: "0193e8c2-7b1a-7c3d-9f2e-1a2b3c4d5e6f",
    },
  });
});

test("extracts filter values after the delimiter", () => {
  expect(getFilterValue("@job_run")).toEqual("@job_run");
  expect(getFilterValue("@job_run:ab12")).toEqual("ab12");
});
