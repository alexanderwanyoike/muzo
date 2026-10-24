import { fileURLToPath } from "node:url";
import { defineConfig } from "eslint/config";
import desktop from "../apps/desktop/eslint.config.js";

export default defineConfig(desktop, {
  files: ["**/*.ts"],
  languageOptions: {
    parserOptions: {
      projectService: true,
      tsconfigRootDir: fileURLToPath(new URL(".", import.meta.url)),
    },
  },
  rules: {
    "import-x/no-extraneous-dependencies": [
      "error",
      {
        devDependencies: true,
        packageDir: fileURLToPath(new URL(".", import.meta.url)),
      },
    ],
  },
});
