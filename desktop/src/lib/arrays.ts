export function last<T>(values: readonly T[]): T | undefined {
  return values[values.length - 1];
}

export function first<T>(values: readonly T[]): T | undefined {
  return values[0];
}
