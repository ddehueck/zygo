export const FILTER_SIGIL = "@";
export const FILTER_DELIMITER = ":";

export function filterPrefix(prefix: string): string {
  return `${FILTER_SIGIL}${prefix}${FILTER_DELIMITER}`;
}

/** Value after the first `:` in a filter string, or the full text when none. */
export function getFilterValue(text: string): string {
  if (text.includes(FILTER_DELIMITER)) {
    return text.slice(text.indexOf(FILTER_DELIMITER) + FILTER_DELIMITER.length);
  }
  return text;
}
