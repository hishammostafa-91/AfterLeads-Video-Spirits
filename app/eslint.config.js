// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// ESLint flat config (docs/02 §2.14): strict type-checked TypeScript,
// react-hooks, jsx-a11y errors, no JSX text literals (G-13), and
// formatjs message hygiene.
import jsxA11y from "eslint-plugin-jsx-a11y";
import formatjs from "eslint-plugin-formatjs";
import i18next from "eslint-plugin-i18next";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: [
      "dist",
      // Generated ts-rs bindings (docs/02 §2.14: generated files are
      // never hand-edited, so they are not lint targets).
      "src/ipc/bindings",
      // This config file is not part of any tsconfig project.
      "eslint.config.js",
    ],
  },
  ...tseslint.configs.strictTypeChecked,
  jsxA11y.flatConfigs.recommended,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
      },
    },
    plugins: {
      "react-hooks": reactHooks,
      i18next,
      formatjs,
    },
    rules: {
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "error",
      "i18next/no-literal-string": "error",
      "formatjs/enforce-id": "error",
      "formatjs/no-invalid-icu": "error",
      "formatjs/enforce-placeholders": "error",
    },
  },
  {
    // Attribute-literal coverage (reviewer M8): kept at the
    // text-children default here because jsx-only mode also flags
    // test fixtures' component props (no per-file rule override
    // applies cleanly); product attribute strings are enforced by the
    // i18n checker's used-but-missing scan and were verified clean.
    files: [
      "src/components/**/*.tsx",
      "src/harness/**/*.tsx",
      "src/i18n/**/*.tsx",
    ],
    rules: {
      "i18next/no-literal-string": "error",
    },
  },
  {
    // Node-run files (no @types/node in the baseline): keep the
    // syntax-level rules, drop the type-aware ones whose node globals
    // they cannot resolve (vite.config.ts precedent). tokens.test.ts
    // and fonts.test.ts run in the node environment and read files
    // via fs/crypto.
    files: [
      "**/*.mts",
      "src/design/tokens.test.ts",
      "src/design/fonts.test.ts",
      "src/design/contrast.test.ts",
    ],
    extends: [tseslint.configs.disableTypeChecked],
  },
);
