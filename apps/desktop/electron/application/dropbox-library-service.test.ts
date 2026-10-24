// @vitest-environment node
import { expect, it, vi } from "vitest";
import { DropboxLibraryService } from "./dropbox-library-service";
import type { LibraryDto } from "../../src/types";

it("registers and syncs a Dropbox library using main-process credentials", async () => {
  const libraries: LibraryDto[] = [];
  const bindings = new Map<string, string>();
  const scan = vi.fn(async () => ({
    ok: true as const,
    value: { tracksScanned: 2 },
  }));
  const service = new DropboxLibraryService(
    {
      add: async (library) => {
        libraries.push(library);
      },
      list: async () => libraries,
      findById: async (id) =>
        libraries.find((library) => library.id === id) ?? null,
    },
    {
      status: async () => ({
        ok: true,
        value: { connected: true, accountId: "account-1" },
      }),
      accessToken: async () => ({ ok: true, value: "main-process-token" }),
    },
    {
      register: async (library, accountId) => {
        libraries.push(library);
        bindings.set(library.id, accountId);
      },
      accountFor: async (id) => bindings.get(id) ?? null,
      intervalMinutes: async () => 5,
      setIntervalMinutes: async () => {},
    },
    { scan },
    () => "lib-1",
  );

  expect(
    await service.add({ name: "Cloud Music", location: "/Music/" }),
  ).toEqual({
    ok: true,
    value: {
      id: "lib-1",
      name: "Cloud Music",
      kind: "dropbox",
      location: "/Music",
    },
  });
  expect(await service.sync("lib-1")).toEqual({
    ok: true,
    value: { tracksScanned: 2 },
  });
  expect(scan).toHaveBeenCalledWith({
    libraryId: "lib-1",
    accessToken: "main-process-token",
  });
  bindings.set("lib-1", "other-account");
  expect(await service.sync("lib-1")).toMatchObject({
    ok: false,
    error: { kind: "notConnected" },
  });
  expect(scan).toHaveBeenCalledTimes(1);
  expect(
    await service.add({ name: "Bad", location: "../../music" }),
  ).toMatchObject({ ok: false, error: { kind: "invalidInput" } });
});

it("shares an in-flight library sync between background polling and manual refresh", async () => {
  const library: LibraryDto = {
    id: "cloud",
    kind: "dropbox",
    name: "Cloud",
    location: "/Music",
  };
  let finish: (report: { tracksScanned: number }) => void = () => {};
  const report = new Promise<{ tracksScanned: number }>((resolve) => {
    finish = resolve;
  });
  const scan = vi.fn(async () => ({ ok: true as const, value: await report }));
  const service = new DropboxLibraryService(
    {
      add: async () => {},
      list: async () => [library],
      findById: async () => library,
    },
    {
      status: async () => ({
        ok: true,
        value: { connected: true, accountId: "account" },
      }),
      accessToken: async () => ({ ok: true, value: "token" }),
    },
    {
      register: async () => {},
      accountFor: async () => "account",
      intervalMinutes: async () => 5,
      setIntervalMinutes: async () => {},
    },
    { scan },
    () => "id",
  );
  const first = service.sync("cloud");
  const second = service.sync("cloud");
  await new Promise<void>((resolve) => {
    setImmediate(resolve);
  });
  expect(scan).toHaveBeenCalledTimes(1);
  finish({ tracksScanned: 2 });
  expect(await Promise.all([first, second])).toEqual([
    { ok: true, value: { tracksScanned: 2 } },
    { ok: true, value: { tracksScanned: 2 } },
  ]);
});

it("binds token retrieval to the library owner when the connected account changes during lookup", async () => {
  let connectedAccount = "account-A";
  let finishLookup: (owner: string) => void = () => {};
  const owner = new Promise<string>((resolve) => {
    finishLookup = resolve;
  });
  const accessToken = vi.fn(async (expectedAccount?: string) =>
    expectedAccount && expectedAccount !== connectedAccount
      ? {
          ok: false as const,
          error: { kind: "notConnected" as const, message: "Account changed" },
        }
      : { ok: true as const, value: "token-B" },
  );
  const library: LibraryDto = {
    id: "cloud",
    kind: "dropbox",
    name: "Cloud",
    location: "/Music",
  };
  const scan = vi.fn(async () => ({
    ok: true as const,
    value: { tracksScanned: 0 },
  }));
  const service = new DropboxLibraryService(
    {
      add: async () => {},
      list: async () => [library],
      findById: async () => library,
    },
    {
      status: async () => ({
        ok: true,
        value: { connected: true, accountId: connectedAccount },
      }),
      accessToken,
    },
    {
      register: async () => {},
      accountFor: () => owner,
      intervalMinutes: async () => 5,
      setIntervalMinutes: async () => {},
    },
    { scan },
    () => "id",
  );
  const syncing = service.sync("cloud");
  await new Promise<void>((resolve) => {
    setImmediate(resolve);
  });
  connectedAccount = "account-B";
  finishLookup("account-A");
  expect(await syncing).toMatchObject({
    ok: false,
    error: { kind: "notConnected" },
  });
  expect(accessToken).toHaveBeenCalledWith("account-A");
  expect(scan).not.toHaveBeenCalled();
});

it("leaves no orphan library when saving its account binding fails", async () => {
  const libraries: LibraryDto[] = [];
  const fail = async () => {
    throw new Error("disk full");
  };
  const settings = {
    register: fail,
    accountFor: async () => null,
    intervalMinutes: async () => 5,
    setIntervalMinutes: async () => {},
  };
  const service = new DropboxLibraryService(
    {
      add: async (library) => {
        libraries.push(library);
      },
      list: async () => libraries,
      findById: async () => null,
    },
    {
      status: async () => ({
        ok: true,
        value: { connected: true, accountId: "account" },
      }),
      accessToken: async () => ({ ok: true, value: "token" }),
    },
    settings,
    { scan: vi.fn() },
    () => "id",
  );
  expect(
    await service.add({ name: "Cloud", location: "/Music" }),
  ).toMatchObject({ ok: false, error: { kind: "storage" } });
  expect(libraries).toEqual([]);
});

it("does not add a folder selected from a different connected account", async () => {
  const libraries: LibraryDto[] = [];
  const service = new DropboxLibraryService(
    {
      add: async () => {},
      list: async () => libraries,
      findById: async () => null,
    },
    {
      status: async () => ({
        ok: true,
        value: { connected: true, accountId: "account-B" },
      }),
      accessToken: async () => ({ ok: true, value: "token" }),
    },
    {
      register: async (library) => {
        libraries.push(library);
      },
      accountFor: async () => "account-B",
      intervalMinutes: async () => 5,
      setIntervalMinutes: async () => {},
    },
    { scan: vi.fn() },
    () => "id",
  );
  expect(
    await service.add({
      name: "Music",
      location: "/Music",
      accountId: "account-A",
    }),
  ).toMatchObject({ ok: false, error: { kind: "notConnected" } });
  expect(libraries).toEqual([]);
});
