export function isPositiveInteger(value: number): boolean {
  return Number.isSafeInteger(value) && value > 0;
}

// Convert a boolean to `1` / `0` for arithmetic
export function boolToInt(value: boolean): 0 | 1 {
  return value ? 1 : 0;
}
