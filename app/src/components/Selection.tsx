// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Selection controls (07 §7.6): Checkbox, Switch, RadioGroup, Radio,
 * Select, ComboBox — all on React Aria Components (the current
 * CheckboxField/CheckboxButton and SwitchField/SwitchButton APIs).
 */
import type { JSX, ReactNode } from "react";
import {
  CheckboxButton,
  CheckboxField,
  type CheckboxFieldProps,
  ComboBox as RACComboBox,
  type ComboBoxProps,
  Input as RACInput,
  Label as RACLabel,
  ListBox as RACListBox,
  ListBoxItem,
  RadioButton,
  RadioField,
  type RadioFieldProps,
  RadioGroup as RACRadioGroup,
  type RadioGroupProps,
  Select as RACSelect,
  type SelectProps,
  SwitchButton,
  SwitchField,
  type SwitchFieldProps,
} from "react-aria-components";
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
      <CheckboxButton className={styles.checkboxButton} />
      {children}
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
      <SwitchButton className={styles.switchButton} />
      {children}
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
      <RadioButton />
      {children}
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
      {label && <RACLabel>{label}</RACLabel>}
      <RACListBox>{children}</RACListBox>
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
      {label && <RACLabel>{label}</RACLabel>}
      <div className={styles.comboBoxRow}>
        <RACInput />
      </div>
      <RACListBox>{children}</RACListBox>
    </RACComboBox>
  );
}

export { ListBoxItem };
