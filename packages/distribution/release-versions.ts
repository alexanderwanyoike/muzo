import { readFile } from "node:fs/promises";
import { parse } from "smol-toml";

const root = new URL("../../", import.meta.url);

async function workspaceVersion(path: string): Promise<unknown> {
  const metadata: unknown = JSON.parse(
    await readFile(new URL(path, root), "utf8"),
  );
  return metadata && typeof metadata === "object" && "version" in metadata
    ? metadata.version
    : undefined;
}

async function nativeVersions() {
  const readToml = async (path: string) =>
    parse(await readFile(new URL(path, root), "utf8"));
  const [workspace, cargo, packager] = await Promise.all([
    workspaceVersion("apps/desktop-rust/package.json"),
    readToml("apps/desktop-rust/Cargo.toml"),
    readToml("apps/desktop-rust/Packager.toml"),
  ]);
  const { package: crate } = cargo;
  return {
    workspace,
    binary:
      crate && typeof crate === "object" && "version" in crate
        ? crate.version
        : undefined,
    packager: packager.version,
  };
}

export async function readReleaseVersions(native: boolean) {
  const [rootVersion, desktopVersion, rust] = await Promise.all([
    workspaceVersion("package.json"),
    workspaceVersion("apps/desktop/package.json"),
    native ? nativeVersions() : undefined,
  ]);
  return { rootVersion, desktopVersion, native: rust };
}
