/** Append `value` to the array at `key`, creating the array if needed. */
export function appendToMap<K, V>(map: Map<K, V[]>, key: K, value: V) {
  const list = map.get(key);
  if (list) {
    list.push(value);
  } else {
    map.set(key, [value]);
  }
}
