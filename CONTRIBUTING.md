# Contributing

Read AGENTS.md and HANDOFF.md before making changes. GPO Word follows the upstream Rust command architecture, layering, crash-prevention and asset-attribution rules.

Keep changes focused on a verifiable document workflow. Use synthetic fixtures in public tests and keep private templates, completed documents, keys and logs containing internal data out of Git.

Project and engineering documentation is written in English. The owner-requested Thai user manual lives under docs/manual/th. Thai remains a supported application and document language. Preserve runtime terminology and Thai regression cases when editing prose.

Run the relevant tests and cargo xtask ci before preparing a completed change. Report unavailable checks explicitly instead of claiming they passed. Include the user-visible behavior, evidence and remaining limitations in the change description.

Every new asset needs an ATTRIBUTION.md entry. Retain upstream notices and licenses. The project logo does not replace official template marks or imply organizational approval.

Owner workflow: prepare reviewable local commits and push commands. The owner will push to siriponsri/gpo-word; do not force-push or publish releases automatically.
