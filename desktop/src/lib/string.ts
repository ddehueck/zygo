export function lastChar(str: string): string {
  return str.slice(-1);
}

export function pluralize(count: number, singular: string, plural: string): string {
  return count === 1 ? singular : plural;
}
