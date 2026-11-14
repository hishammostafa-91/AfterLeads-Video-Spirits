// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * ColorPicker (07 §7.6): the panel color picker — an HSV square
 * (saturation × brightness, RAC ColorArea), a hue slider, a hex field
 * plus RGB channel fields, and a recent-colors swatch row. RAC's
 * color primitives provide the interaction semantics; the hex string
 * is the component's value boundary. The eyedropper lands with the
 * viewer integration of a later package (spec fence).
 */
import type { JSX } from "react";
import { FormattedMessage, useIntl } from "react-intl";
import {
  ColorArea as RACColorArea,
  ColorField as RACColorField,
  ColorSlider as RACColorSlider,
  ColorSwatch,
  ColorSwatchPicker,
  ColorSwatchPickerItem,
  ColorThumb,
  Input as RACInput,
  Label as RACLabel,
  parseColor,
} from "react-aria-components";
import type { Color } from "react-aria-components";
import { NumberField } from "./NumberField";
import styles from "./ColorPicker.module.css";

export interface ColorPickerProps {
  /** The current color as a hex string, e.g. "#ff8800". */
  value: string;
  onChange: (hex: string) => void;
  /** Recently used colors (hex), offered as swatches. */
  recent?: readonly string[];
  className?: string;
}

export function ColorPicker({
  value,
  onChange,
  recent = [],
  className,
}: ColorPickerProps): JSX.Element {
  const { formatMessage } = useIntl();
  const color = parseColor(value);
  const classes = [styles.picker, className].filter(Boolean).join(" ");
  const change = (next: Color | null): void => {
    if (next === null) return;
    onChange(next.toString("hex"));
  };
  return (
    <div className={classes}>
      <RACColorArea
        colorSpace="hsb"
        xChannel="saturation"
        yChannel="brightness"
        value={color}
        onChange={change}
        aria-label={formatMessage({
          id: "components.color.saturationBrightness",
        })}
        className={styles.area}
      >
        <ColorThumb className={styles.thumb} />
      </RACColorArea>
      <RACColorSlider
        channel="hue"
        value={color.toFormat("hsb")}
        onChange={change}
        className={styles.hue}
      >
        {/* The Label child is RAC's naming path for a ColorSlider:
            its id becomes the thumb input's aria-labelledby. A plain
            aria-label would stay on the group and leave the input
            unnamed (RAC 1.22.1 / react-aria 3.53.1). */}
        <RACLabel className={styles.sliderLabel}>
          <FormattedMessage id="components.color.hue" />
        </RACLabel>
        <ColorThumb className={styles.thumb} />
      </RACColorSlider>
      <div className={styles.fields}>
        <div className={styles.hexField}>
          <RACColorField
            value={color}
            onChange={change}
            aria-label={formatMessage({ id: "components.color.hex" })}
          >
            <RACInput className={styles.hexInput} />
          </RACColorField>
        </div>
        <NumberField
          label={formatMessage({ id: "components.color.red" })}
          value={color.getChannelValue("red")}
          minValue={0}
          maxValue={255}
          step={1}
          onChange={(channel) => {
            change(color.withChannelValue("red", channel));
          }}
        />
        <NumberField
          label={formatMessage({ id: "components.color.green" })}
          value={color.getChannelValue("green")}
          minValue={0}
          maxValue={255}
          step={1}
          onChange={(channel) => {
            change(color.withChannelValue("green", channel));
          }}
        />
        <NumberField
          label={formatMessage({ id: "components.color.blue" })}
          value={color.getChannelValue("blue")}
          minValue={0}
          maxValue={255}
          step={1}
          onChange={(channel) => {
            change(color.withChannelValue("blue", channel));
          }}
        />
      </div>
      {recent.length > 0 && (
        <ColorSwatchPicker
          value={color}
          onChange={change}
          aria-label={formatMessage({ id: "components.color.recent" })}
          className={styles.recents}
        >
          {recent.map((hex) => (
            <ColorSwatchPickerItem key={hex} color={hex}>
              <ColorSwatch />
            </ColorSwatchPickerItem>
          ))}
        </ColorSwatchPicker>
      )}
    </div>
  );
}
