// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The i18n runtime (REQ-I18N-001; research D4): the six shipped locale
 * catalogs statically imported into one registry (offline-safe,
 * constitution VII), a locale store whose updates re-render the
 * provider and set `dir`/`lang` through `direction.ts` — a language
 * switch never reloads. The default resolves the Windows display
 * language (`navigator.languages`) to a shipped locale, else English.
 */
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { create } from "zustand";
import ar from "../locales/ar.json";
import de from "../locales/de.json";
import en from "../locales/en.json";
import es from "../locales/es.json";
import fr from "../locales/fr.json";
import ptBr from "../locales/pt-BR.json";
import { applyDirection } from "./direction";

export const MESSAGES = {
  en,
  ar,
  fr,
  es,
  de,
  "pt-BR": ptBr,
} as const;

export type LocaleTag = keyof typeof MESSAGES;
export const LOCALE_TAGS = Object.keys(MESSAGES) as LocaleTag[];

function isLocaleTag(value: string): value is LocaleTag {
  return value in MESSAGES;
}

/** Resolve the Windows display language to a shipped locale, else `en`. */
export function resolveDefaultLocale(preferred: readonly string[]): LocaleTag {
  for (const candidate of preferred) {
    if (isLocaleTag(candidate)) return candidate;
    const base = candidate.split("-")[0] ?? candidate;
    const match = LOCALE_TAGS.find(
      (tag) => tag === candidate || tag.split("-")[0] === base,
    );
    if (match) return match;
  }
  return "en";
}

interface LocaleState {
  locale: LocaleTag;
  setLocale: (locale: LocaleTag) => void;
}

export const useLocaleStore = create<LocaleState>((set) => ({
  locale: "en",
  setLocale: (locale) => {
    set({ locale });
    applyDirection(locale);
  },
}));

/** The provider wrapping the app/harness; re-renders on language switch.
 * The optional override is harness-only pseudo-locale rendering
 * (LOC-90/91; tests render through the same provider). */
export function I18n({
  children,
  override,
}: {
  children: ReactNode;
  override?: { locale: LocaleTag; messages: Record<string, string> };
}): JSX.Element {
  const storeLocale = useLocaleStore((state) => state.locale);
  const locale = override?.locale ?? storeLocale;
  const messages = override?.messages ?? MESSAGES[storeLocale];
  return (
    <IntlProvider locale={locale} defaultLocale="en" messages={messages}>
      {children}
    </IntlProvider>
  );
}
