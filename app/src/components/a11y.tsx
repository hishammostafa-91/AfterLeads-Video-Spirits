// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Accessibility infrastructure (07 §7.13 rule 3; REQ-A11Y-002
 * components share): one polite live region for edits and job
 * completions, one assertive region for errors. Screens and
 * components announce through {@link announceStatus} /
 * {@link announceError}; the regions are mounted once at the harness
 * root.
 */
import { useIntl } from "react-intl";
import type { JSX } from "react";

const STATUS_REGION_ID = "alvs-live-status";
const ERROR_REGION_ID = "alvs-live-errors";

/** Mount the two live regions (once, at the harness root). */
export function LiveRegions(): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <>
      <div
        id={STATUS_REGION_ID}
        aria-live="polite"
        aria-label={formatMessage({ id: "a11y.statusRegion" })}
      />
      <div
        id={ERROR_REGION_ID}
        aria-live="assertive"
        role="alert"
        aria-label={formatMessage({ id: "a11y.errorsRegion" })}
      />
    </>
  );
}

function announce(regionId: string, message: string): void {
  const region = document.getElementById(regionId);
  if (region) {
    // Clear-then-set so identical messages re-announce (07 §7.13).
    region.textContent = "";
    window.setTimeout(() => {
      region.textContent = message;
    }, 0);
  }
}

/** Announce an edit or job completion politely. */
export function announceStatus(message: string): void {
  announce(STATUS_REGION_ID, message);
}

/** Announce an error assertively. */
export function announceError(message: string): void {
  announce(ERROR_REGION_ID, message);
}
