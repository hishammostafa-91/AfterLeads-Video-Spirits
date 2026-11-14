// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * KeyframeNavigator (07 §7.6): ◀ ◆ ▶ — jump to the previous keyframe,
 * toggle the one at the playhead, jump to the next; ◀/▶ disable at
 * the respective boundary (never color alone: disabled state also
 * drops the button from the tab order).
 */
import type { JSX } from "react";
import { ChevronFirst, ChevronLast, Diamond } from "../design/icons";
import { useIntl } from "react-intl";
import { Button } from "./Button";
import styles from "./Keyframe.module.css";

export interface KeyframeNavigatorProps {
  /** A keyframe exists before the playhead (enables ◀). */
  hasPrevious: boolean;
  /** A keyframe exists after the playhead (enables ▶). */
  hasNext: boolean;
  /** Whether the playhead sits on a keyframe (◆ state). */
  isOnKeyframe: boolean;
  onPrevious: () => void;
  onNext: () => void;
  /** Add (true) or remove (false) the keyframe at the playhead. */
  onToggle: (add: boolean) => void;
  className?: string;
}

export function KeyframeNavigator({
  hasPrevious,
  hasNext,
  isOnKeyframe,
  onPrevious,
  onNext,
  onToggle,
  className,
}: KeyframeNavigatorProps): JSX.Element {
  const { formatMessage } = useIntl();
  const classes = [styles.navigator, className].filter(Boolean).join(" ");
  let toggleMessageId = "components.keyframe.add";
  if (isOnKeyframe) toggleMessageId = "components.keyframe.remove";
  return (
    <div className={classes} role="group">
      <Button
        variant="ghost"
        aria-label={formatMessage({ id: "components.keyframe.prev" })}
        isDisabled={!hasPrevious}
        className={styles.navButton}
        onPress={onPrevious}
      >
        <ChevronFirst size={16} aria-hidden="true" />
      </Button>
      <Button
        variant="ghost"
        aria-pressed={isOnKeyframe}
        aria-label={formatMessage({ id: toggleMessageId })}
        className={styles.navButton}
        onPress={() => {
          onToggle(!isOnKeyframe);
        }}
      >
        <Diamond
          size={12}
          aria-hidden="true"
          className={isOnKeyframe ? styles.diamondOn : undefined}
          fill={isOnKeyframe ? "currentColor" : "none"}
        />
      </Button>
      <Button
        variant="ghost"
        aria-label={formatMessage({ id: "components.keyframe.next" })}
        isDisabled={!hasNext}
        className={styles.navButton}
        onPress={onNext}
      >
        <ChevronLast size={16} aria-hidden="true" />
      </Button>
    </div>
  );
}
