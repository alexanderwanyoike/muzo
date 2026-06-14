import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import App from "./App";

describe("App", () => {
  it("renders the Muzo heading", () => {
    const { getByRole } = render(<App />);
    expect(getByRole("heading", { name: "Muzo" })).toBeDefined();
  });
});
