// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Selection controls (07 §7.6): Checkbox, Switch, RadioGroup, Radio,
 * Select, ComboBox — all on React Aria Components (the current
 * CheckboxField/CheckboxButton and SwitchField/SwitchButton APIs).
 */
import type { JSX, ReactNode } from "react";
import {
  Button as RACButton,
  CheckboxButton,
  CheckboxField,
  type CheckboxFieldProps,
  ComboBox as RACComboBox,
  type ComboBoxProps,
  Input as RACInput,
  Label as RACLabel,
  ListBox as RACListBox,
  ListBoxItem,
  Popover as RACPopover,
  RadioButton,
  RadioField,
  type RadioFieldProps,
  RadioGroup as RACRadioGroup,
  type RadioGroupProps,
  Select as RACSelect,
  type SelectProps,
  SelectValue,
  SwitchButton,
  SwitchField,
  type SwitchFieldProps,
} from "react-aria-components";
import { ChevronDown } from "lucide-react";
import styles from "./Selection.module.css";

function selectionClass(base: string, className: string | undefined): string {
  return [base, className].filter(Boolean).join(" ");
}

export interface CheckboxProps extends Omit<CheckboxFieldProps, "className"> {
  className?: string;
  children?: ReactNode;
}

export function Checkbox({
  className,
  children,
  ...props
}: CheckboxProps): JSX.Element {
  return (
    <CheckboxField
      {...props}
      className={selectionClass(styles.checkbox, className)}
    >
      <CheckboxButton className={styles.checkboxButton}>
        <span className={styles.checkIndicator} aria-hidden="true" />
        {children}
      </CheckboxButton>
    </CheckboxField>
  );
}

export interface SwitchProps extends Omit<SwitchFieldProps, "className"> {
  className?: string;
  children?: ReactNode;
}

export function Switch({
  className,
  children,
  ...props
}: SwitchProps): JSX.Element {
  return (
    <SwitchField
      {...props}
      className={selectionClass(styles.switch, className)}
    >
      <SwitchButton className={styles.switchButton}>
        <span className={styles.switchTrack} aria-hidden="true">
          <span className={styles.switchThumb} />
        </span>
        {children}
      </SwitchButton>
    </SwitchField>
  );
}

export interface RadioGroupWrapperProps extends Omit<
  RadioGroupProps,
  "className"
> {
  className?: string;
  children?: ReactNode;
}

export function RadioGroup({
  className,
  children,
  ...props
}: RadioGroupWrapperProps): JSX.Element {
  return (
    <RACRadioGroup
      {...props}
      className={selectionClass(styles.radioGroup, className)}
    >
      {children}
    </RACRadioGroup>
  );
}

export interface RadioWrapperProps extends Omit<RadioFieldProps, "className"> {
  className?: string;
  children?: ReactNode;
}

export function Radio({
  className,
  children,
  ...props
}: RadioWrapperProps): JSX.Element {
  return (
    <RadioField {...props} className={selectionClass(styles.radio, className)}>
      <RadioButton className={styles.radioButton}>
        <span className={styles.radioIndicator} aria-hidden="true" />
        {children}
      </RadioButton>
    </RadioField>
  );
}

export interface SelectWrapperProps extends Omit<
  SelectProps,
  "children" | "className"
> {
  label?: string;
  children?: ReactNode;
  className?: string;
}

export function Select({
  label,
  children,
  className,
  ...props
}: SelectWrapperProps): JSX.Element {
  return (
    <RACSelect {...props} className={selectionClass(styles.select, className)}>
      {label && <RACLabel className={styles.label}>{label}</RACLabel>}
      <RACButton className={styles.trigger}>
        <SelectValue className={styles.triggerValue} />
        <ChevronDown size={16} aria-hidden="true" />
      </RACButton>
      <RACPopover className={styles.popover}>
        <RACListBox className={styles.options}>{children}</RACListBox>
      </RACPopover>
    </RACSelect>
  );
}

export interface ComboBoxWrapperProps extends Omit<
  Partial<ComboBoxProps<object>>,
  "children" | "className"
> {
  label?: string;
  children?: ReactNode;
  className?: string;
}

export function ComboBox({
  label,
  children,
  className,
  ...props
}: ComboBoxWrapperProps): JSX.Element {
  return (
    <RACComboBox
      {...props}
      className={selectionClass(styles.comboBox, className)}
    >
      {label && <RACLabel className={styles.label}>{label}</RACLabel>}
      <div className={styles.comboBoxRow}>
        <RACInput className={styles.input} />
        <RACButton className={styles.comboBoxButton}>
          <ChevronDown size={16} aria-hidden="true" />
        </RACButton>
      </div>
      <RACPopover className={styles.popover}>
        <RACListBox className={styles.options}>{children}</RACListBox>
      </RACPopover>
    </RACComboBox>
  );
}

export { ListBoxItem };
