---
description: Fill missing translations in ar, fr, es, de, pt-BR with the i18n subagent.
---
Use the i18n subagent to add every key that exists in app/src/locales/en.json but is missing or outdated in the other five locale files, following docs/07-experience.md §7.12 and app/src/locales/en.meta.json. Then run `pnpm -C app i18n:check` and report the keys changed per locale. $ARGUMENTS
