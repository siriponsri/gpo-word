# Build and package on Windows

Use the home/development PC for compilation. Teammates using a built application do not need Rust or C++ development tools.

## Prerequisites

- Rust 1.95 or later, installed through rustup.
- Microsoft MSVC C++ Build Tools, the Desktop development with C++ workload and Windows SDK.
- Sufficient disk space and access to Rust dependency downloads.
- A normal Windows user profile for DPAPI verification.

Official setup: https://learn.microsoft.com/en-us/windows/dev-environment/rust/setup

Detect tools before reinstalling them. If the shell was open during installation, close it and start a new terminal. VS Code may also need restarting before it inherits the new PATH.

## Checks and build

Run from the folder containing Cargo.toml:

```powershell
cargo --version
rustc --version
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown
cargo test -p wordcraft-engine --test gpo
```

If the targeted tests pass:

```powershell
cargo xtask ci
```

If the required checks pass:

```powershell
cargo build --release -p wordcraft -p wordcraft-cli
```

The GPO additions are uncompiled source in this handoff. Treat errors as work to diagnose, not evidence that a dependency should be bypassed or a test disabled.

## PowerShell launchers

The upstream preparation encountered Windows' disabled-script policy. For a trusted local script on a machine you administer, a process-scoped launch can use:

```powershell
powershell -NoProfile -ExecutionPolicy RemoteSigned -File .\scripts\Start-GPOWord.ps1
```

If a downloaded, trusted script is marked as originating from the Internet, inspect it first and unblock that specific file if appropriate:

```powershell
Unblock-File -LiteralPath .\scripts\Start-GPOWord.ps1
```

Organization policy can override local policy. Use organization approval for a work PC; do not change machine-wide settings to evade its controls.

## Package

Extract the private template pack so docsStorage/catalog.json exists under the project, or pass -Templates with its absolute directory.

After build and smoke-test success:

```powershell
powershell -NoProfile -ExecutionPolicy RemoteSigned -File .\scripts\Package-Portable.ps1
```

The default dist/GPOWord-portable folder must not already exist. For a later build, pass a fresh -OutputDirectory rather than overwriting a deployed copy.

Verify that the ZIP contains GPOWord.exe, docsStorage, GPOWord.portable, licenses and empty user folders. Test from another folder as a standard user. Current-user DPAPI keys do not travel between users.

## Public Windows CI

The GPO Windows preview workflow builds code without private templates or keys. Its public artifact is not the private team package. On a successful run, download the artifact from the GitHub Actions run page.

The workflow is prepared source until pushed and executed. No successful build artifact is implied by a link or filename.
