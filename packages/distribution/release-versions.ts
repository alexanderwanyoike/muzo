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

async function readToml(path: string) {
  return parse(await readFile(new URL(path, root), "utf8"));
}

export function readPackagerConfig() {
  return readToml("apps/desktop-rust/Packager.toml");
}

async function nativeVersions() {
  const [workspace, cargo, packager] = await Promise.all([
    workspaceVersion("apps/desktop-rust/package.json"),
    readToml("apps/desktop-rust/Cargo.toml"),
    readPackagerConfig(),
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

export async function readReleaseVersions() {
  const [rootVersion, native] = await Promise.all([
    workspaceVersion("package.json"),
    nativeVersions(),
  ]);
  return { rootVersion, native };
}
