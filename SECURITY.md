# Security and private data

Do not put API keys, DPAPI blobs, private templates, completed GPO documents or identifiable example content in issues, commits, screenshots or public build artifacts.

The intended API flow sends only explicitly reviewed text to the configured endpoint. Current-user Windows DPAPI protects persisted keys; portable data is still account-specific. Never copy an encrypted key to another teammate as a way to share credentials.

Report a suspected exposure privately to the repository owner through a verified channel. Do not create a public issue containing the affected value. If a key is exposed, revoke it with the provider and remove the exposure from active files; history cleanup requires explicit review.

Document checks assist authors. Reference syntax, model output and watermark settings do not establish authorization, approval or controlled-copy status.
