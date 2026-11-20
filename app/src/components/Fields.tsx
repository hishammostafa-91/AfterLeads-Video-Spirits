// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Text fields (07 §7.6): SearchField, TextField, TextArea. User
 * content uses `dir="auto"` for bidi (07 §7.11). Composed on React
 * Aria's TextField/SearchField so the label is associated with the
 * input and disabled/invalid/required state propagates (FR-012).
 */
import type { JSX } from "react";
import { useId } from "react";
import {
  Input,
  Label,
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
      {label && <Label className={styles.label}>{label}</Label>}
      <Input className={styles.input} placeholder={placeholder} dir="auto" />
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
      {label && <Label className={styles.label}>{label}</Label>}
      <Input className={styles.input} placeholder={placeholder} dir="auto" />
    </RACSearchField>
  );
}

export interface TextAreaProps {
  label?: string;
  placeholder?: string;
  className?: string;
  value?: string;
  defaultValue?: string;
  onChange?: (value: string) => void;
  isDisabled?: boolean;
  /** Error state: marks the field invalid (07 §7.6 error stories). */
  isInvalid?: boolean;
  isRequired?: boolean;
  name?: string;
}

export function TextArea({
  label,
  placeholder,
  className,
  value,
  defaultValue,
  onChange,
  ...props
}: TextAreaProps): JSX.Element {
  const id = useId();
  return (
    <div className={classes(className)}>
      {label && (
        <Label className={styles.label} htmlFor={id}>
          {label}
        </Label>
      )}
      <textarea
        id={id}
        className={styles.textarea}
        placeholder={placeholder}
        dir="auto"
        disabled={props.isDisabled}
        data-invalid={props.isInvalid ? "" : undefined}
        aria-invalid={props.isInvalid || undefined}
        required={props.isRequired}
        name={props.name}
        value={value}
        defaultValue={defaultValue}
        onChange={
          onChange
            ? (event) => {
                onChange(event.target.value);
              }
            : undefined
        }
      />
    </div>
  );
}
