type SortKey = string | number | bigint;

function compareSortKeys(a: SortKey, b: SortKey): number {
  if (typeof a === "string" && typeof b === "string") {
    return a.localeCompare(b);
  }
  if (a < b) return -1;
  if (a > b) return 1;
  return 0;
}

/**
 * Build a comparator that sorts by each selector in order.
 *
 * @example
 * items.sort(genCompare(
 *   (item) => item.created_at,
 *   (item) => item.id,
 * ));
 */
export function genCompareFn<T>(...selectors: Array<(item: T) => SortKey>): (a: T, b: T) => number {
  return (a, b) => {
    for (const select of selectors) {
      const result = compareSortKeys(select(a), select(b));
      if (result !== 0) return result;
    }
    return 0;
  };
}
