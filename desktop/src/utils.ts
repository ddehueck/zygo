export function assertNever(value: never): never {
  throw new Error(`Unhandled value: ${JSON.stringify(value)}`);
}

class InvariantViolation extends Error {
  constructor(message: string) {
    super(message);
    this.name = "InvariantViolation";
  }
}

export function invariant(condition: boolean, message?: string): asserts condition {
  if (!condition) {
    throw new InvariantViolation(message ?? "Invariant violation");
  }
}
