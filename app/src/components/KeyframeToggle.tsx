// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * KeyframeToggle (07 §7.6): the stopwatch button that adds or removes
 * a keyframe at the playhead — `aria-pressed` carries the state and
 * the accessible name switches between add/remove.
 */
import type { JSX } from "react";
import { Timer } from "../design/icons";
import { useIntl } from "react-intl";
import { Button } from "./Button";
import styles from "./Keyframe.module.css";

export interface KeyframeToggleProps {
  /** Whether the property already has a keyframe at the playhead. */
  isKeyframed: boolean;
  /** Add (true) or remove (false) the keyframe at the playhead. */
  onToggle: (add: boolean) => void;
  className?: string;
}

export function KeyframeToggle({
  isKeyframed,
  onToggle,
  className,
}: KeyframeToggleProps): JSX.Element {
  const { formatMessage } = useIntl();
  const classes = [styles.toggle, className].filter(Boolean).join(" ");
  let messageId = "components.keyframe.add";
  if (isKeyframed) messageId = "components.keyframe.remove";
  return (
    <Button
      variant="ghost"
      aria-pressed={isKeyframed}
      aria-label={formatMessage({ id: messageId })}
      className={classes}
      onPress={() => {
        onToggle(!isKeyframed);
      }}
    >
      <Timer size={16} aria-hidden="true" />
    </Button>
  );
}
