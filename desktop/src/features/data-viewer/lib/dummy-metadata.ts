// Placeholder sizes until object-store metadata is available.
export function dummySizeBytes(id: number): number {
  const sizes = [48_128, 256_000, 1_248_768, 12_582_912, 98_304_000, 1_073_741_824, 42_100_000_000];
  return sizes[Math.abs(id) % sizes.length]!;
}

export function formatByteSize(bytes: number): string {
  if (bytes < 1000) return `${bytes} B`;
  if (bytes < 1_000_000) return `${(bytes / 1000).toFixed(bytes < 10_000 ? 1 : 0)} KB`;
  if (bytes < 1_000_000_000) return `${(bytes / 1_000_000).toFixed(bytes < 10_000_000 ? 1 : 0)} MB`;
  return `${(bytes / 1_000_000_000).toFixed(bytes < 10_000_000_000 ? 1 : 0)} GB`;
}
