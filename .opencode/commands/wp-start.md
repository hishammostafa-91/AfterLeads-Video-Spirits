---
description: Start a work package on a new branch created from up-to-date main (docs/10-execution.md §10.4.3). Usage /wp-start WP-NN
---
Start work package $ARGUMENTS. Follow these steps exactly and stop at the first failure.

1. Run `git status --porcelain`. If it prints anything, list the files and stop: the tree must be clean.
2. In docs/10-execution.md §10.7 find the card whose heading starts with "### $ARGUMENTS". If none exists, stop and report.
3. From the card read: the Branch line (`<type>/<NNN>-<slug>`), "Depends on", "Implements", and the test IDs in "Acceptance".
4. Open STATUS.md. Every work package listed in "Depends on" must have state `done`. If not, stop and list the missing ones.
5. Run `pwsh -File tools/git/New-WorkBranch.ps1 -Type <type> -Number <NNN> -Slug <slug>`.
6. In STATUS.md set the row of $ARGUMENTS to state `in-progress`, branch `<type>/<NNN>-<slug>`, today's date, next step `specify`. Commit with `git commit -s -m "chore(docs): start $ARGUMENTS"`.
7. Print this single line for the owner to paste (fill in the brackets from the card):

/speckit.specify SPECIFY_FEATURE_DIRECTORY=specs/<NNN>-<slug> Implement $ARGUMENTS exactly as specified by its card in docs/10-execution.md §10.7. Requirements: <IDs from Implements>. Acceptance tests: <T IDs>. Read only the sections in the card's Read line. Derive every requirement from docs/01-product.md and cite IDs; add nothing that is not in the canonical documents.

8. Stop. Do not write a spec or code in this command.
