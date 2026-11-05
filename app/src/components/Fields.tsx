// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Text fields (07 §7.6): SearchField, TextField, TextArea. User
 * content uses `dir="auto"` for bidi (07 §7.11).
 */
import type { JSX } from "react";
import {
  SearchField as RACSearchField,
  type SearchFieldProps as RACSearchFieldProps,
  TextField as RACTextField,
  type TextFieldProps as RACTextFieldProps,
} from "react-aria-components";
import styles from "./Fields.module.css";

function classes(className: string | undefined): string {
  return [styles.field, className].filter(Boolean).join(" ");
}

export interface TextFieldProps extends Omit<RACTextFieldProps, "className"> {
  label?: string;
  placeholder?: string;
  className?: string;
}

export function TextField({
  label,
  placeholder,
  className,
  ...props
}: TextFieldProps): JSX.Element {
  return (
    <RACTextField {...props} className={classes(className)}>
      {label && <label className={styles.label}>{label}</label>}
      <input className={styles.input} placeholder={placeholder} dir="auto" />
    </RACTextField>
  );
}

export interface SearchFieldProps extends Omit<
  RACSearchFieldProps,
  "className"
> {
  label?: string;
  placeholder?: string;
  className?: string;
}

export function SearchField({
  label,
  placeholder,
  className,
  ...props
}: SearchFieldProps): JSX.Element {
  return (
    <RACSearchField {...props} className={classes(className)}>
      {label && <label className={styles.label}>{label}</label>}
      <input className={styles.input} placeholder={placeholder} dir="auto" />
    </RACSearchField>
  );
}

export interface TextAreaProps {
  label?: string;
  placeholder?: string;
  className?: string;
  value?: string;
  defaultValue?: string;
  isDisabled?: boolean;
  isRequired?: boolean;
  name?: string;
}

export function TextArea({
  label,
  placeholder,
  className,
  ...props
}: TextAreaProps): JSX.Element {
  return (
    <label className={classes(className)}>
      {label && <span className={styles.label}>{label}</span>}
      <textarea
        className={styles.textarea}
        placeholder={placeholder}
        dir="auto"
        disabled={props.isDisabled}
        required={props.isRequired}
        name={props.name}
        defaultValue={props.defaultValue}
      />
    </label>
  );
}
