// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The UI harness shell (07 §7.6 stories; research D7): renders the
 * story grid with mode selection from the URL
 * (?theme=dark|light|high-contrast&locale=<tag>&density=comfortable|compact)
 * so one build serves every Playwright project. Landmarks per 07
 * §7.13 rule 2; the live regions per rule 3 mount here. The theme
 * store and the i18n runtime (US1/US2) take over their halves as they
 * land; until then the shell applies URL modes directly.
 */
import { FormattedMessage, IntlProvider } from "react-intl";
import type { JSX } from "react";
import ar from "../locales/ar.json";
import de from "../locales/de.json";
import en from "../locales/en.json";
import es from "../locales/es.json";
import fr from "../locales/fr.json";
import ptBr from "../locales/pt-BR.json";
import { LiveRegions } from "../components/a11y";
import styles from "./HarnessApp.module.css";

const MESSAGES = { en, ar, fr, es, de, "pt-BR": ptBr } as const;
export type HarnessLocale = keyof typeof MESSAGES;

export function harnessParams(): {
  theme: "dark" | "light" | "high-contrast";
  locale: HarnessLocale;
  density: "comfortable" | "compact";
} {
  const query = new URLSearchParams(window.location.search);
  const themeParam = query.get("theme");
  const localeParam = query.get("locale");
  const densityParam = query.get("density");
  const theme =
    themeParam === "light" || themeParam === "high-contrast"
      ? themeParam
      : "dark";
  const locale = (
    localeParam && localeParam in MESSAGES ? localeParam : "en"
  ) as HarnessLocale;
  const density = densityParam === "compact" ? "compact" : "comfortable";
  return { theme, locale, density };
}

/** Apply the URL modes to the document root (US1/US2 stores own this later). */
function applyModes(): void {
  const { theme, locale, density } = harnessParams();
  document.documentElement.dataset.theme = theme;
  document.documentElement.lang = locale;
  document.documentElement.dir = locale === "ar" ? "rtl" : "ltr";
  document.documentElement.dataset.density = density;
}

export function HarnessApp(): JSX.Element {
  applyModes();
  const { locale } = harnessParams();
  return (
    <IntlProvider
      locale={locale}
      defaultLocale="en"
      messages={MESSAGES[locale]}
    >
      <main className={`container ${styles.shell}`}>
        <header className={styles.header} role="banner">
          <h1 className={styles.title}>
            <FormattedMessage id="harness.title" />
          </h1>
          <p className={styles.subtitle}>
            <FormattedMessage id="harness.subtitle" />
          </p>
        </header>
        <div className={styles.toolbar} role="toolbar">
          {/* Theme/accent/zoom/locale controls arrive with US1/US2. */}
        </div>
        <section className={styles.stories} aria-label="Stories">
          <h2 className={styles.storiesTitle}>
            <FormattedMessage id="harness.stories.title" />
          </h2>
          <p>
            <FormattedMessage id="harness.stories.empty" />
          </p>
        </section>
        <LiveRegions />
      </main>
    </IntlProvider>
  );
}
