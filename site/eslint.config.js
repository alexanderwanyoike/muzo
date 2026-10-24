import { fileURLToPath } from "node:url";
import { defineConfig } from "eslint/config";
import { configs, plugins } from "eslint-config-airbnb-extended";
import prettier from "eslint-config-prettier/flat";
import globals from "globals";

export default defineConfig(
  { ignores: ["dist/**", "node_modules/**"] },
  plugins.stylistic,
  plugins.importX,
  plugins.typescriptEslint,
  ...configs.base.all,
  { languageOptions: { globals: globals.node } },
  {
    files: ["**/*.ts"],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: fileURLToPath(new URL(".", import.meta.url)),
      },
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-unsafe-assignment": "error",
      "@typescript-eslint/no-unsafe-argument": "error",
      "@typescript-eslint/no-unsafe-call": "error",
      "@typescript-eslint/no-unsafe-member-access": "error",
      "@typescript-eslint/no-unsafe-return": "error",
    },
  },
  {
    rules: {
      // Node 22 supports native for-of without a transpiler runtime.
      "no-restricted-syntax": [
        "error",
        "ForInStatement",
        "LabeledStatement",
        "WithStatement",
      ],
      "import-x/no-extraneous-dependencies": [
        "error",
        {
          devDependencies: true,
          packageDir: [
            fileURLToPath(new URL(".", import.meta.url)),
            fileURLToPath(new URL("..", import.meta.url)),
          ],
        },
      ],
    },
  },
  prettier,
);
