// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Numeric-control stories (07 §7.6; T021–T024): NumberField, Slider,
 * AngleDial, TimecodeField, KeyframeToggle/Navigator.
 */
import { FormattedMessage, useIntl } from "react-intl";
import type { JSX } from "react";
import { useState } from "react";
import { NumberField } from "../../components/NumberField";
import { AngleDial, Slider } from "../../components/Slider";
import { TimecodeField } from "../../components/TimecodeField";
import { KeyframeToggle } from "../../components/KeyframeToggle";
import { KeyframeNavigator } from "../../components/KeyframeNavigator";
import styles from "./stories.module.css";
import type { StoryFile } from "./types";

const FPS = 24;

function Scale(): JSX.Element {
  const { formatMessage } = useIntl();
  const [value, setValue] = useState(100);
  return (
    <div className={styles.column}>
      <NumberField
        label={formatMessage({ id: "components.number.scaleLabel" })}
        value={value}
        minValue={0}
        maxValue={400}
        step={1}
        suffix="%"
        onChange={setValue}
      />
      <NumberField
        label={formatMessage({ id: "components.number.mixedLabel" })}
        isMixed
        suffix="%"
      />
    </div>
  );
}

function OpacitySlider(): JSX.Element {
  const [value, setValue] = useState(0.6);
  return (
    <Slider
      label={<FormattedMessage id="components.slider.opacityLabel" />}
      minValue={0}
      maxValue={1}
      step={0.05}
      value={value}
      onChange={setValue}
    />
  );
}

function Dial(): JSX.Element {
  const { formatMessage } = useIntl();
  const [angle, setAngle] = useState(45);
  return (
    <div className={styles.row}>
      <AngleDial
        value={angle}
        defaultValue={0}
        onChange={setAngle}
        label={formatMessage({ id: "components.slider.angleLabel" })}
      />
      <NumberField
        label={formatMessage({ id: "components.slider.angleLabel" })}
        value={angle}
        minValue={0}
        maxValue={359}
        onChange={(v) => {
          setAngle(v);
        }}
        suffix="°"
      />
    </div>
  );
}

function Timecode(): JSX.Element {
  const [frames, setFrames] = useState(FPS * 62 + 11);
  return <TimecodeField value={frames} onChange={setFrames} fps={FPS} />;
}

function Keyframes(): JSX.Element {
  const [isKeyframed, setIsKeyframed] = useState(false);
  return (
    <div className={styles.row}>
      <KeyframeToggle isKeyframed={isKeyframed} onToggle={setIsKeyframed} />
      <KeyframeNavigator
        hasPrevious
        hasNext
        isOnKeyframe={isKeyframed}
        onPrevious={() => {}}
        onNext={() => {}}
        onToggle={setIsKeyframed}
      />
    </div>
  );
}

export const numericStories: StoryFile = {
  id: "numeric",
  stories: [
    { id: "number-field.scale-mixed", render: () => <Scale /> },
    { id: "slider.opacity", render: () => <OpacitySlider /> },
    { id: "slider.angle-dial", render: () => <Dial /> },
    { id: "timecode.field", render: () => <Timecode /> },
    { id: "keyframe.toggle-navigator", render: () => <Keyframes /> },
  ],
};
