export function assertNever(value: never): never {
  throw new Error(`Unhandled value: ${JSON.stringify(value)}`);
}

class InvariantViolation extends Error {
  constructor(message: string) {
    super(message);
    this.name = "InvariantViolation";
  }
}

/**
 * Throws an error if the condition is false. This function is used to enforce
 * invariants in the code, and it will throw an `InvariantViolation` error if
 * the condition is not met.
 */
export function invariant(condition: boolean, message?: string): asserts condition {
  if (!condition) {
    throw new InvariantViolation(message ?? "Invariant violation");
  }
}

export type Maybe<T> = T | undefined | null;

export function errMsg(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }
  return String(err);
}
