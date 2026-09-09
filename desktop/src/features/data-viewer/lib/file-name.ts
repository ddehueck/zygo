export function getFileNameFromUri(uri: string): string {
  const withoutQuery = uri.split("?")[0] ?? uri;
  const parts = withoutQuery.split("/").filter(Boolean);
  return parts[parts.length - 1] ?? uri;
}

export function getFileKind(uri: string): string {
  const fileName = getFileNameFromUri(uri);
  const extension = fileName.includes(".") ? (fileName.split(".").pop()?.toLowerCase() ?? "") : "";

  switch (extension) {
    case "json":
    case "jsonl":
    case "ndjson":
      return "JSON";
    case "parquet":
      return "Parquet";
    case "csv":
    case "tsv":
      return "CSV";
    case "arrow":
    case "feather":
      return "Arrow";
    case "txt":
    case "log":
      return "Text";
    case "png":
    case "jpg":
    case "jpeg":
    case "webp":
      return "Image";
    case "":
      return "File";
    default:
      return extension.toUpperCase();
  }
}
