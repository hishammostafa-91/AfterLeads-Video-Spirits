// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Text-field stories (07 §7.6; T027 backfill).
 */
import { useIntl } from "react-intl";
import type { JSX } from "react";
import { SearchField, TextArea, TextField } from "../../components/Fields";
import type { StoryFile } from "./types";

function Text(): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <TextField
      label={formatMessage({ id: "components.fields.nameLabel" })}
      placeholder={formatMessage({ id: "components.fields.namePlaceholder" })}
    />
  );
}

function Search(): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <SearchField
      label={formatMessage({ id: "components.fields.searchLabel" })}
      placeholder={formatMessage({ id: "components.fields.searchPlaceholder" })}
    />
  );
}

function Notes(): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <TextArea
      label={formatMessage({ id: "components.fields.notesLabel" })}
      placeholder={formatMessage({ id: "components.fields.notesPlaceholder" })}
    />
  );
}

export const fieldsStories: StoryFile = {
  id: "fields",
  stories: [
    { id: "fields.text", render: () => <Text /> },
    { id: "fields.search", render: () => <Search /> },
    { id: "fields.textarea", render: () => <Notes /> },
  ],
};
