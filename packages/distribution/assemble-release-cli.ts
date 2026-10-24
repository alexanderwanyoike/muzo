import { parseArgs } from "node:util";
import { assembleRelease } from "./assemble-release";
import {
  distributionFailure,
  type DistributionResult,
} from "./distribution-result";

async function run(): Promise<DistributionResult<string>> {
  try {
    const { values } = parseArgs({
      options: {
        input: { type: "string" },
        output: { type: "string" },
        version: { type: "string" },
        tag: { type: "string" },
        "source-commit": { type: "string" },
      },
    });
    if (
      !values.input ||
      !values.output ||
      !values.version ||
      !values.tag ||
      !values["source-commit"]
    )
      return distributionFailure(
        "invalidRelease",
        "Provide input, output, version, tag and source-commit.",
      );
    const result = await assembleRelease({
      inputDirectory: values.input,
      outputDirectory: values.output,
      version: values.version,
      tag: values.tag,
      sourceCommit: values["source-commit"],
    });
    return result.ok
      ? { ok: true, value: `Release set ready: ${values.output}` }
      : result;
  } catch {
    return distributionFailure(
      "invalidRelease",
      "Could not read release assembly options.",
    );
  }
}
const result = await run();
if (result.ok) process.stdout.write(`${result.value}\n`);
else {
  process.stderr.write(`${result.error.message}\n`);
  process.exitCode = 1;
}
