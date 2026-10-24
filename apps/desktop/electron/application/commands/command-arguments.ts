export function parseInput(
  args: unknown,
  command: string,
): Record<string, unknown> {
  if (!args || typeof args !== "object" || !("input" in args)) {
    throw { kind: "repository", message: `missing ${command} input` };
  }

  const input = (args as { input: unknown }).input;
  if (!input || typeof input !== "object") {
    throw { kind: "repository", message: `invalid ${command} input` };
  }

  return input as Record<string, unknown>;
}

export function parseNonEmptyString(
  input: Record<string, unknown>,
  field: string,
  command: string,
): string {
  const value = input[field];
  if (typeof value !== "string" || value.trim().length === 0) {
    throw { kind: "repository", message: `invalid ${command} ${field}` };
  }
  return value;
}

export function parseStringArray(
  input: Record<string, unknown>,
  field: string,
  command: string,
): string[] {
  const value = input[field];
  if (
    !Array.isArray(value) ||
    value.some((item) => typeof item !== "string" || item.trim().length === 0)
  ) {
    throw { kind: "repository", message: `invalid ${command} ${field}` };
  }
  return value;
}

export function parseNullableString(
  input: Record<string, unknown>,
  field: string,
  command: string,
): string | null {
  const value = input[field];
  if (value === null) {
    return null;
  }
  if (typeof value !== "string") {
    throw { kind: "repository", message: `invalid ${command} ${field}` };
  }
  return value;
}

export function parseNullableNumber(
  input: Record<string, unknown>,
  field: string,
  command: string,
): number | null {
  const value = input[field];
  if (value === null) {
    return null;
  }
  if (typeof value !== "number") {
    throw { kind: "repository", message: `invalid ${command} ${field}` };
  }
  return value;
}
