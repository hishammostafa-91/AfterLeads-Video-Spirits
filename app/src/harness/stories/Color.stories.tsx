// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Color + curve stories (07 §7.6; T025–T026).
 */
import type { JSX } from "react";
import { useState } from "react";
import { ColorPicker } from "../../components/ColorPicker";
import { ColorWheel } from "../../components/ColorWheel";
import {
  CurveEditor,
  DEFAULT_CURVE,
  type Curve,
} from "../../components/CurveEditor";
import { PointControl } from "../../components/PointControl";
import styles from "./stories.module.css";
import type { StoryFile } from "./types";

function Picker(): JSX.Element {
  const [hex, setHex] = useState("#ff8800");
  return (
    <ColorPicker
      value={hex}
      onChange={setHex}
      recent={["#ff8800", "#0033cc", "#22cc88"]}
    />
  );
}

function Wheel(): JSX.Element {
  const [hex, setHex] = useState("#3366cc");
  return <ColorWheel value={hex} defaultValue="#3366cc" onChange={setHex} />;
}

function Curve(): JSX.Element {
  const [curve, setCurve] = useState<Curve>(DEFAULT_CURVE);
  return (
    <div className={styles.column}>
      <CurveEditor value={curve} onChange={setCurve} />
      <PointControl
        x={50}
        y={50}
        onX={() => {}}
        onY={() => {}}
        handle="smooth"
        onHandle={() => {}}
      />
    </div>
  );
}

export const colorStories: StoryFile = {
  id: "color",
  stories: [
    { id: "color.picker", render: () => <Picker /> },
    { id: "color.wheel", render: () => <Wheel /> },
    { id: "curves.editor", render: () => <Curve /> },
  ],
};
