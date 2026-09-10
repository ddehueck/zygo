import { revealItemInDir } from "@tauri-apps/plugin-opener";

export function localPathFromUri(uri: string) {
  if (uri.startsWith("file://")) {
    try {
      return decodeURIComponent(new URL(uri).pathname);
    } catch {
      return null;
    }
  }

  return uri.includes("://") ? null : uri;
}

export async function revealLocalUri(uri: string) {
  const path = localPathFromUri(uri);
  if (!path) return false;

  try {
    await revealItemInDir(path);
    return true;
  } catch {
    return false;
  }
}
