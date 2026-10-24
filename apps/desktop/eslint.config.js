import { fileURLToPath } from "node:url";
import { defineConfig } from "eslint/config";
import { configs, plugins } from "eslint-config-airbnb-extended";
import prettier from "eslint-config-prettier/flat";
import globals from "globals";

const typescriptFiles = ["**/*.{ts,tsx}"];

export default defineConfig(
  {
    ignores: [
      "dist/**",
      "dist-electron/**",
      "out/**",
      "node_modules/**",
      "vite.config.js",
      "vite.config.d.ts",
    ],
  },
  plugins.stylistic,
  plugins.importX,
  plugins.typescriptEslint,
  plugins.react,
  plugins.reactA11y,
  plugins.reactHooks,
  ...configs.base.all,
  ...configs.react.all,
  {
    files: typescriptFiles,
    languageOptions: {
      ecmaVersion: 2022,
      parserOptions: {
        projectService: true,
        tsconfigRootDir: fileURLToPath(new URL(".", import.meta.url)),
      },
    },
    rules: {
      "@typescript-eslint/no-use-before-define": [
        "error",
        { functions: false, typedefs: false, ignoreTypeReferences: true },
      ],
      "class-methods-use-this": "off",
      // Typed unions provide exhaustive switch coverage without unreachable defaults.
      "default-case": "off",
      "consistent-return": "off",
      "@typescript-eslint/switch-exhaustiveness-check": [
        "error",
        { considerDefaultExhaustiveForUnions: true },
      ],
      "@typescript-eslint/class-methods-use-this": [
        "error",
        { ignoreClassesThatImplementAnInterface: true },
      ],
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-unsafe-assignment": "error",
      "@typescript-eslint/no-unsafe-argument": "error",
      "@typescript-eslint/no-unsafe-call": "error",
      "@typescript-eslint/no-unsafe-member-access": "error",
      "@typescript-eslint/no-unsafe-return": "error",
    },
  },
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: { globals: globals.browser },
  },
  {
    files: ["electron/**/*.ts", "*.{js,cjs,ts}"],
    languageOptions: { globals: globals.node },
  },
  {
    rules: {
      // Named exports match the application's ports and cohesive adapter modules.
      "import-x/prefer-default-export": "off",
      // React 18 uses the automatic JSX runtime and parameter defaults.
      "react/react-in-jsx-scope": "off",
      "react/jsx-uses-react": "off",
      "react/require-default-props": [
        "error",
        { ignoreFunctionalComponents: true },
      ],
      // Lifecycle effects reset loading/editor state. React Compiler is not enabled.
      "react-hooks/set-state-in-effect": "off",
      // ARIA in HTML permits a native button to host a focusable splitter.
      "jsx-a11y/no-interactive-element-to-noninteractive-role": [
        "error",
        { tr: ["none", "presentation"], button: ["separator"] },
      ],
      "react/jsx-no-bind": [
        "error",
        { allowArrowFunctions: true, allowFunctions: true },
      ],
      "jsx-a11y/label-has-associated-control": ["error", { depth: 3 }],
      "no-void": ["error", { allowAsStatement: true }],
      // ES2022 supports native for-of without transpiler runtime overhead.
      "no-restricted-syntax": [
        "error",
        "ForInStatement",
        "LabeledStatement",
        "WithStatement",
      ],
      "no-underscore-dangle": ["error", { allow: ["__MUZO_RUNTIME__"] }],
      "import-x/no-extraneous-dependencies": [
        "error",
        {
          packageDir: [
            fileURLToPath(new URL(".", import.meta.url)),
            fileURLToPath(new URL("../..", import.meta.url)),
          ],
          devDependencies: [
            "**/*.test.{ts,tsx}",
            "**/*config.{js,cjs,ts}",
            "**/electron/main.ts",
            "**/electron/preload.cjs",
            "**/electron/build-electron-main.ts",
            "**/src/test-runtime.ts",
            "**/fixtures/**/*.ts",
          ],
        },
      ],
    },
  },
  {
    files: ["electron/**/*.ts"],
    // Scans, paginated APIs and migrations intentionally sequence I/O.
    rules: { "no-await-in-loop": "off", "no-continue": "off" },
  },
  {
    files: [
      "src/audio-player-engine.ts",
      "electron/infrastructure/filesystem/node-filesystem-library-watcher.ts",
    ],
    // Audio engines, React refs and watcher registrations are imperative handles.
    rules: {
      "no-param-reassign": [
        "error",
        {
          props: true,
          ignorePropertyModificationsFor: [
            "audio",
            "currentTrackIdRef",
            "registration",
          ],
        },
      ],
    },
  },
  {
    files: ["electron/infrastructure/logging/console-logger.ts"],
    rules: { "no-console": ["error", { allow: ["error"] }] },
  },
  {
    files: ["**/*.test.{ts,tsx}", "src/test-runtime.ts"],
    // Test fixtures and Vitest asymmetric matchers intentionally accept arbitrary values.
    rules: {
      "max-classes-per-file": "off",
      "@typescript-eslint/class-methods-use-this": "off",
      "@typescript-eslint/no-use-before-define": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-return": "off",
    },
  },
  {
    files: ["**/*.tsx"],
    // Conditional JSX views remain together rather than becoming render-only wrappers.
    rules: { "no-nested-ternary": "off" },
  },
  {
    files: ["electron/ipc/legacy-result-command-adapter.ts"],
    // The IPC compatibility transport serializes typed failure records.
    rules: { "@typescript-eslint/only-throw-error": "off" },
  },
  prettier,
);
