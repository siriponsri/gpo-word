# Local development and private reference documents

The intended owner workspace is Desktop/myProject/gpo-word. Authoritative private reference documents remain in the sibling Desktop/myProject/gpoDocs/docsStorage directory.

The provided Install-Desktop.ps1 copies a prepared project without overwriting differing existing files. It preserves unrelated samples and merges .gitignore rules, including samples/private. It does not initialize or push a remote repository.

This preparation session cannot write to the Desktop destination because sandbox approval is disabled. The project is staged in the authorized workspace and the installer was tested against a harmless fixture. Do not state that the actual Desktop destination was updated.

The Thai user manual is an explicitly requested exception to the English engineering-documentation policy. Capture screenshots from the running application using synthetic documents. Use WORDCRAFT_PROFILE_DIR for isolated UI runs; the upstream control protocol supports authenticated ui.screenshot and ui.click without touching other applications.

The local build uses a workspace-contained Rust GNU toolchain and MinGW, with one compile job and debug symbols disabled for development/tests because the host has limited available RAM. This is a build-resource choice, not a change to document behavior or a reduction in test assertions.

No API keys or previous user data belong in the distributable package. Keep docsStorage and completed examples private. The owner will push public source to siriponsri/gpo-word.
