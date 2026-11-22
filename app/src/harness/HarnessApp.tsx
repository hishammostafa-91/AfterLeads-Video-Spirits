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
import { FormattedMessage, useIntl } from "react-intl";
import type { JSX, ReactNode } from "react";
import { useState } from "react";
import { LiveRegions } from "../components/a11y";
import {
  setThemePersistence,
  useThemeStore,
  type Accent,
  type ThemeName,
} from "../design/theme";
import {
  I18n,
  LOCALE_TAGS,
  useLocaleStore,
  type LocaleTag,
} from "../i18n/runtime";
import { pseudoBase, pseudoCatalog } from "../i18n/pseudo";
import { storyFiles } from "./stories";
import { setWorkspacePersistence } from "../shell/dock/workspaces";
import { localStorageWorkspacePersistence } from "./mock/workspacePort";
import { setIpcTransport } from "../ipc/transport";
import { ipcMockTransport } from "./mock/ipcMock";
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
  locale: LocaleTag | "en-XA" | "ar-XB";
  density: "comfortable" | "compact";
  /** Render only the story with this id (?story=). */
  story: string | null;
} {
  const query = new URLSearchParams(window.location.search);
  const themeParam = query.get("theme");
  const localeParam = query.get("locale");
  const densityParam = query.get("density");
  const theme: ThemeName =
    themeParam === "light" || themeParam === "high-contrast"
      ? themeParam
      : "dark";
  const locale =
    localeParam === "en-XA" || localeParam === "ar-XB"
      ? localeParam
      : ((localeParam && localeParam in MESSAGES
          ? localeParam
          : "en") as LocaleTag);
  const density = densityParam === "compact" ? "compact" : "comfortable";
  const story = query.get("story");
  return { theme, locale, density, story };
}

/** Bind the theme persistence seam to the URL (research D12), once. */
let urlPersistenceBound = false;
function ensureUrlPersistence(initial: { theme: ThemeName }): void {
  if (urlPersistenceBound) return;
  urlPersistenceBound = true;
  void setWorkspacePersistence(
    localStorageWorkspacePersistence(window.localStorage),
  );
  setIpcTransport(ipcMockTransport());
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
  const { locale, setLocale } = useLocaleStore();
  const [density, setDensity] = useState<"comfortable" | "compact">(
    harnessParams().density,
  );
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
      <label className={styles.control}>
        <span className={styles.controlLabel}>
          <FormattedMessage id="harness.toolbar.locale" />
        </span>
        <select
          value={locale}
          onChange={(event) => {
            setLocale(event.target.value as LocaleTag);
          }}
        >
          {LOCALE_TAGS.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      </label>
      <label className={styles.control}>
        <span className={styles.controlLabel}>
          <FormattedMessage id="harness.toolbar.density" />
        </span>
        <select
          value={density}
          onChange={(event) => {
            const value = event.target.value as "comfortable" | "compact";
            document.documentElement.dataset.density = value;
            setDensity(value);
          }}
        >
          {(["comfortable", "compact"] as const).map((option) => (
            <option key={option} value={option}>
              {option === "comfortable" ? (
                <FormattedMessage id="harness.toolbar.density.comfortable" />
              ) : (
                <FormattedMessage id="harness.toolbar.density.compact" />
              )}
            </option>
          ))}
        </select>
      </label>
    </>
  );
}

function themeLabel(theme: ThemeName): JSX.Element {
  switch (theme) {
    case "dark":
      return <FormattedMessage id="harness.toolbar.theme.dark" />;
    case "light":
      return <FormattedMessage id="harness.toolbar.theme.light" />;
    case "high-contrast":
      return <FormattedMessage id="harness.toolbar.theme.highContrast" />;
  }
}

function accentLabel(accent: Accent): JSX.Element {
  switch (accent) {
    case "clr-06":
      return <FormattedMessage id="harness.toolbar.accent.blue" />;
    case "clr-07":
      return <FormattedMessage id="harness.toolbar.accent.purple" />;
    case "clr-04":
      return <FormattedMessage id="harness.toolbar.accent.green" />;
    case "clr-02":
      return <FormattedMessage id="harness.toolbar.accent.orange" />;
    case "clr-08":
      return <FormattedMessage id="harness.toolbar.accent.pink" />;
  }
}

/**
 * The stories landmark (07 §7.13 rule 2). A child of <I18n> so the
 * accessible name localizes (G-13) — the provider wraps this inside
 * HarnessApp's return, so useIntl must live here, not in HarnessApp.
 */
function StoriesSection(props: { children: ReactNode }): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <section
      className={styles.stories}
      aria-label={formatMessage({ id: "harness.stories.title" })}
    >
      {props.children}
    </section>
  );
}

export function HarnessApp(): JSX.Element {
  const params = harnessParams();
  ensureUrlPersistence({ theme: params.theme });
  ensureLocaleFromUrl(
    params.locale === "en-XA" || params.locale === "ar-XB"
      ? pseudoBase(params.locale)
      : params.locale,
  );
  document.documentElement.dataset.density = params.density;
  // Pseudo-locales render through the override (tests only; 12 §12.14).
  const override =
    params.locale === "en-XA"
      ? { locale: "en" as const, messages: pseudoCatalog("en-XA", MESSAGES.en) }
      : params.locale === "ar-XB"
        ? {
            locale: "ar" as const,
            messages: pseudoCatalog("ar-XB", MESSAGES.ar),
          }
        : undefined;
  return (
    <I18n override={override}>
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
        <StoriesSection>
          <h2 className={styles.storiesTitle}>
            <FormattedMessage id="harness.stories.title" />
          </h2>
          <div className={styles.storyGrid}>
            {storyFiles
              .flatMap((file) => file.stories)
              .filter(
                (story) => params.story === null || story.id === params.story,
              )
              .map((story) => (
                <div
                  key={story.id}
                  className={
                    story.wide
                      ? `${styles.storyCard} ${styles.storyCardWide}`
                      : styles.storyCard
                  }
                  data-story-id={story.id}
                >
                  <span className={styles.storyLabel}>{story.id}</span>
                  {story.render()}
                </div>
              ))}
          </div>
        </StoriesSection>
        <LiveRegions />
      </main>
    </I18n>
  );
}

/** Initialize the locale store from the URL once (tests reset it). */
let localeInitialized = false;
function ensureLocaleFromUrl(locale: LocaleTag): void {
  if (localeInitialized) return;
  localeInitialized = true;
  useLocaleStore.getState().setLocale(locale);
}
