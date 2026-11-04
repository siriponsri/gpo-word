# GPO Word — portable preview and development handoff

1. Extract gpo-word.zip.
2. Run `GPOWord.exe` from the extracted folder. Read the Thai guide at `docs/manual/th/index.html`.
3. The template pack is already included under docsStorage. No second ZIP is needed.
4. For development, open the extracted folder in VS Code. Cargo.toml and HANDOFF.md are directly inside it.
5. Review VALIDATION.md for the tests and limitations. The owner controls any push to https://github.com/siriponsri/gpo-word.

## Short prompt

Read AGENTS.md, HANDOFF.md, GOAL.md, PLAN.md, VALIDATION.md and docs/CODEX-PROMPT.md, then complete the continuation task. Work until the Windows proof of concept is built, checked and packaged. Keep all repository documentation in English and preserve Thai application/document support. Use the supplied GPO Word brand with accurate attribution. Fix real build and workflow failures rather than merely describing them. The private template pack is already in docsStorage and must remain out of Git. Prepare local commits and precise owner push commands; do not push or publish automatically. Report actual test evidence and remaining limitations.

## Contents and boundary

This local ZIP combines the Windows preview executable, source, documentation, project logo and private template pack. It contains no Git history, completed memorandum bodies or API keys.

The supplied private pack has 16 template entries: 15 copied source templates and one memorandum-header preset. Dedicated OJT and TOR source templates are still missing.

docsStorage is gitignored. Initialize or open Git only in the gpo-word folder and inspect staged files before the owner pushes public code. Do not use a forced Git add to include private templates.

The preview has passed selected synthetic Windows checks. Full CI, representative template fidelity and deployment on another account or machine still require validation.
