import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import AddLibraryForm from "./AddLibraryForm";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type { LibraryDto } from "./types";

const mockedInvoke = invoke as unknown as ReturnType<typeof vi.fn>;
const mockedOpenDialog = openDialog as unknown as ReturnType<typeof vi.fn>;

beforeEach(() => {
  mockedInvoke.mockReset();
  mockedOpenDialog.mockReset();
});

describe("AddLibraryForm", () => {
  it("renders a name field, a location field, a browse button, and a submit button", () => {
    render(<AddLibraryForm onAdded={() => {}} />);

    expect(screen.getByLabelText(/name/i)).toBeDefined();
    expect(screen.getByLabelText(/location/i)).toBeDefined();
    expect(screen.getByRole("button", { name: /browse/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /add library/i })).toBeDefined();
  });

  it("calls invoke with the form values when submitted", async () => {
    const user = userEvent.setup();
    const onAdded = vi.fn();
    const created: LibraryDto = {
      id: "lib-1",
      name: "My Music",
      kind: "filesystem",
      location: "/home/user/Music",
    };
    mockedInvoke.mockResolvedValueOnce(created);

    render(<AddLibraryForm onAdded={onAdded} />);

    await user.type(screen.getByLabelText(/name/i), "My Music");
    await user.type(screen.getByLabelText(/location/i), "/home/user/Music");
    await user.click(screen.getByRole("button", { name: /add library/i }));

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledTimes(1));
    expect(mockedInvoke).toHaveBeenCalledWith("add_library", {
      input: {
        name: "My Music",
        kind: "filesystem",
        location: "/home/user/Music",
      },
    });
    expect(onAdded).toHaveBeenCalledWith(created);
  });

  it("opens a folder picker when Browse is clicked and fills the location", async () => {
    const user = userEvent.setup();
    mockedOpenDialog.mockResolvedValueOnce("/home/user/Picked");

    render(<AddLibraryForm onAdded={() => {}} />);

    await user.click(screen.getByRole("button", { name: /browse/i }));

    expect(
      (screen.getByLabelText(/location/i) as HTMLInputElement).value,
    ).toBe("/home/user/Picked");
  });

  it("shows an error when the folder picker cannot open", async () => {
    const user = userEvent.setup();
    mockedOpenDialog.mockRejectedValueOnce(new Error("dialog unavailable"));

    render(<AddLibraryForm onAdded={() => {}} />);

    await user.click(screen.getByRole("button", { name: /browse/i }));

    await waitFor(() =>
      expect(screen.getByText(/could not open the folder picker/i)).toBeDefined(),
    );
  });

  it("shows an error message when invoke rejects", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockRejectedValueOnce({ kind: "emptyName" });

    render(<AddLibraryForm onAdded={() => {}} />);

    await user.type(screen.getByLabelText(/location/i), "/home/user/Music");
    await user.click(screen.getByRole("button", { name: /add library/i }));

    await waitFor(() =>
      expect(
        screen.getByText(/library name must not be empty/i),
      ).toBeDefined(),
    );
  });
});
