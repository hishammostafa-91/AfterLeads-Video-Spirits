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
    // Attribute-literal coverage (round-2 M-04): jsx-only mode also
    // validates JSX attributes in the product components. The plugin
    // (6.1.5) has no file-ignore option (checked its schema and
    // settings handling), so the test fixtures opt out through the
    // later config block — flat-config layering, not a rule option.
    // The jsx-attributes exclude list is the plugin's own defaults
    // plus component-API props whose values are enum identifiers
    // (variant names, roles, dir), not user-facing copy.
    files: ["src/components/**/*.tsx", "src/i18n/**/*.tsx"],
    rules: {
      "i18next/no-literal-string": [
        "error",
        {
          mode: "jsx-only",
          "jsx-attributes": {
            include: [],
            exclude: [
              "className",
              "styleName",
              "style",
              "type",
              "key",
              "id",
              "width",
              "height",
              "variant",
              "kind",
              "size",
              "density",
              "dir",
              "role",
              "fill",
              "strokeWidth",
              "as",
              "name",
              "lang",
              "translate",
              "aria-hidden",
              "slot",
              "locale",
              "defaultLocale",
              "selectedKey",
              "defaultSelectedKey",
            ],
          },
        },
      ],
    },
  },
  {
    // Test fixtures render copy from the catalogs like the app, but
    // they also assert literal marker strings — out of scope for
    // G-13 (never shipped to users).
    files: ["src/**/*.test.tsx", "src/**/*.test.ts"],
    rules: {
      "i18next/no-literal-string": "off",
    },
  },
);
