import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export function locateSqlWasm(file: string): string {
  const packagedPath = join(dirname(fileURLToPath(import.meta.url)), file);
  if (existsSync(packagedPath)) {
    return packagedPath;
  }

  const require = createRequire(import.meta.url);
  return require.resolve(`sql.js/dist/${file}`);
}
