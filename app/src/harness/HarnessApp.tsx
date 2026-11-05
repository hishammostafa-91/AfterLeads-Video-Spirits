// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The UI harness shell (07 §7.6 stories; research D7): renders the
 * story grid with mode selection from the URL
 * (?theme=dark|light|high-contrast&locale=<tag>&density=comfortable|compact)
 * so one build serves every Playwright project. The toolbar (US1)
 * switches theme/accent/zoom live through the theme store with a URL
 * persistence seam (research D12); the i18n runtime (US2) owns the
 * locale half next. Landmarks per 07 §7.13 rule 2; live regions per
 * rule 3.
 */
import { FormattedMessage, IntlProvider } from "react-intl";
import type { JSX } from "react";
import { LiveRegions } from "../components/a11y";
import {
  setThemePersistence,
  useThemeStore,
  type Accent,
  type ThemeName,
} from "../design/theme";
import ar from "../locales/ar.json";
import de from "../locales/de.json";
import en from "../locales/en.json";
import es from "../locales/es.json";
import fr from "../locales/fr.json";
import ptBr from "../locales/pt-BR.json";
import styles from "./HarnessApp.module.css";

const MESSAGES = { en, ar, fr, es, de, "pt-BR": ptBr } as const;
export type HarnessLocale = keyof typeof MESSAGES;

const THEMES = ["dark", "light", "high-contrast"] as const;
const ACCENTS = ["clr-06", "clr-07", "clr-04", "clr-02", "clr-08"] as const;

export function harnessParams(): {
  theme: ThemeName;
  locale: HarnessLocale;
  density: "comfortable" | "compact";
} {
  const query = new URLSearchParams(window.location.search);
  const themeParam = query.get("theme");
  const localeParam = query.get("locale");
  const densityParam = query.get("density");
  const theme: ThemeName =
    themeParam === "light" || themeParam === "high-contrast"
      ? themeParam
      : "dark";
  const locale = (
    localeParam && localeParam in MESSAGES ? localeParam : "en"
  ) as HarnessLocale;
  const density = densityParam === "compact" ? "compact" : "comfortable";
  return { theme, locale, density };
}

/** Bind the theme persistence seam to the URL (research D12), once. */
let urlPersistenceBound = false;
function ensureUrlPersistence(initial: { theme: ThemeName }): void {
  if (urlPersistenceBound) return;
  urlPersistenceBound = true;
  const apply = (): void => {
    const url = new URL(window.location.href);
    const { theme, accent, zoom } = useThemeStore.getState();
    url.searchParams.set("theme", theme);
    url.searchParams.set("accent", accent);
    url.searchParams.set("zoom", String(zoom));
    window.history.replaceState(null, "", url);
  };
  setThemePersistence({
    read: () => {
      const query = new URLSearchParams(window.location.search);
      const accent = query.get("accent");
      const zoom = Number(query.get("zoom"));
      return {
        theme: initial.theme,
        accent: ACCENTS.find((a) => a === accent),
        zoom: Number.isFinite(zoom) && zoom !== 0 ? zoom : undefined,
      };
    },
    write: () => {
      apply();
    },
  });
}

function ToolbarControls(): JSX.Element {
  const { theme, accent, zoom, setTheme, setAccent, setZoom } = useThemeStore();
  return (
    <>
      <label className={styles.control}>
        <span className={styles.controlLabel}>
          <FormattedMessage id="harness.toolbar.theme" />
        </span>
        <select
          value={theme}
          onChange={(event) => {
            setTheme(event.target.value as ThemeName);
          }}
        >
          {THEMES.map((option) => (
            <option key={option} value={option}>
              {themeLabel(option)}
            </option>
          ))}
        </select>
      </label>
      <label className={styles.control}>
        <span className={styles.controlLabel}>
          <FormattedMessage id="harness.toolbar.accent" />
        </span>
        <select
          value={accent}
          onChange={(event) => {
            setAccent(event.target.value as Accent);
          }}
        >
          {ACCENTS.map((option) => (
            <option key={option} value={option}>
              {accentLabel(option)}
            </option>
          ))}
        </select>
      </label>
      <label className={styles.control}>
        <span className={styles.controlLabel}>
          <FormattedMessage
            id="harness.toolbar.zoom"
            values={{ value: zoom }}
          />
        </span>
        <input
          type="range"
          min={80}
          max={200}
          step={10}
          value={zoom}
          onChange={(event) => {
            setZoom(Number(event.target.value));
          }}
        />
      </label>
    </>
  );
}

function themeLabel(theme: ThemeName): JSX.Element {
  const id =
    theme === "dark"
      ? "harness.toolbar.theme.dark"
      : theme === "light"
        ? "harness.toolbar.theme.light"
        : "harness.toolbar.theme.highContrast";
  return <FormattedMessage id={id} />;
}

function accentLabel(accent: Accent): JSX.Element {
  const id = `harness.toolbar.accent.${accentToColor(accent)}`;
  return <FormattedMessage id={id} />;
}

function accentToColor(accent: Accent): string {
  return {
    "clr-06": "blue",
    "clr-07": "purple",
    "clr-04": "green",
    "clr-02": "orange",
    "clr-08": "pink",
  }[accent];
}

export function HarnessApp(): JSX.Element {
  const params = harnessParams();
  ensureUrlPersistence({ theme: params.theme });
  document.documentElement.lang = params.locale;
  document.documentElement.dir = params.locale === "ar" ? "rtl" : "ltr";
  document.documentElement.dataset.density = params.density;
  return (
    <IntlProvider
      locale={params.locale}
      defaultLocale="en"
      messages={MESSAGES[params.locale]}
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
          <ToolbarControls />
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
