import { expect, test, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class {
    onmessage = () => {};
  },
}));
vi.mock("@/bindings", () => ({ commands: {} }));

import { logsPageStaleTime } from "./fetch-logs-page";

test("caches forever only when before_id's exclusive range is fully historical", () => {
  expect(logsPageStaleTime(null, { observed_through_id: 100 })).toBe(0);
  expect(logsPageStaleTime(50, undefined)).toBe(0);
  expect(logsPageStaleTime(50, { observed_through_id: 48 })).toBe(0);
  expect(logsPageStaleTime(50, { observed_through_id: 49 })).toBe(Infinity);
  expect(logsPageStaleTime(50, { observed_through_id: 100 })).toBe(Infinity);
});
