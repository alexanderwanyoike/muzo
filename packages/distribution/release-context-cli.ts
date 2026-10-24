import { appendFile, readFile } from "node:fs/promises";
import { parseArgs } from "node:util";
import {
  distributionFailure,
  type DistributionResult,
} from "./distribution-result";
import { releaseContext } from "./release-context";

async function run(): Promise<DistributionResult<string>> {
  try {
    const { values } = parseArgs({
      options: {
        "source-commit": { type: "string" },
        "github-output": { type: "string" },
      },
    });
    const root = new URL("../../", import.meta.url);
    async function version(path: string): Promise<unknown> {
      const metadata: unknown = JSON.parse(
        await readFile(new URL(path, root), "utf8"),
      );
      return metadata && typeof metadata === "object" && "version" in metadata
        ? metadata.version
        : undefined;
    }
    const result = releaseContext({
      rootVersion: await version("package.json"),
      desktopVersion: await version("apps/desktop/package.json"),
      sourceCommit: values["source-commit"] ?? process.env.GITHUB_SHA,
      tag:
        process.env.GITHUB_REF_TYPE === "tag"
          ? process.env.GITHUB_REF_NAME
          : undefined,
    });
    if (!result.ok) return result;
    if (values["github-output"])
      await appendFile(
        values["github-output"],
        `version=${result.value.version}\ntag=${result.value.tag}\nsourceCommit=${result.value.sourceCommit}\n`,
      );
    return { ok: true, value: JSON.stringify(result.value) };
  } catch {
    return distributionFailure(
      "storage",
      "Could not read the release context or write its verified outputs.",
    );
  }
}
const result = await run();
if (result.ok) process.stdout.write(`${result.value}\n`);
else {
  process.stderr.write(`${result.error.message}\n`);
  process.exitCode = 1;
}
