import { failure, success, type Result } from "../result";

export function parseInput(args: unknown, command: string): Result<Record<string, unknown>> {
  if (!args || typeof args !== "object" || !("input" in args)) return failure("repository", `missing ${command} input`);
  if (!args.input || typeof args.input !== "object" || Array.isArray(args.input)) return failure("repository", `invalid ${command} input`);
  return success(args.input as Record<string, unknown>);
}

export function parseNonEmptyString(input: Record<string, unknown>, field: string, command: string): Result<string> {
  const value = input[field];
  return typeof value === "string" && value.trim() ? success(value) : failure("repository", `invalid ${command} ${field}`);
}

export function parseStringArray(input: Record<string, unknown>, field: string, command: string): Result<string[]> {
  const value = input[field];
  if (!Array.isArray(value) || !value.every((item: unknown): item is string => typeof item === "string" && item.trim().length > 0)) return failure("repository", `invalid ${command} ${field}`);
  return success(value);
}

export function parseNullableString(input: Record<string, unknown>, field: string, command: string): Result<string | null> {
  const value = input[field];
  return value === null || typeof value === "string" ? success(value) : failure("repository", `invalid ${command} ${field}`);
}

export function parseNullableNumber(input: Record<string, unknown>, field: string, command: string): Result<number | null> {
  const value = input[field];
  return value === null || typeof value === "number" ? success(value) : failure("repository", `invalid ${command} ${field}`);
}
