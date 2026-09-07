export function isPositiveInteger(value: number): boolean {
  return Number.isSafeInteger(value) && value > 0;
}
