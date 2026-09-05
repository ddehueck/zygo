export type Result<T, E> = { success: true; data: T } | { success: false; error: E };

export function isOk<T, E>(result: Result<T, E>): result is { success: true; data: T } {
  return result.success;
}

export function isErr<T, E>(result: Result<T, E>): result is { success: false; error: E } {
  return !result.success;
}

export function ok<T, E>(data: T): Result<T, E> {
  return { success: true, data };
}

export function err<T, E>(error: E): Result<T, E> {
  return { success: false, error };
}
