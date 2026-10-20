# Releasing

> **Last updated:** 2026-10-20

A release is a **draft GitHub Release** built by [`.github/workflows/release.yml`](../.github/workflows/release.yml)
on every push to the `release` branch. Publishing the draft is a human step.

## Steps

1. Set the version (the one source of truth is `[workspace.package] version` in `Cargo.toml`):

   ```sh
   cargo xtask version set 0.1.2-beta     # also refreshes Cargo.lock
   cargo xtask version                    # prints it
   ```

   A pre-release tag (`-beta`, `-rc.1`) marks the GitHub Release as a pre-release.
2. Pass the quality gates and commit: `cargo xtask ci`, then add a line to `ROADMAP.md`'s
   progress log.
3. Push `main`, then the same commit to `release`:

   ```sh
   git push origin main
   git push origin main:release
   ```

4. Actions ▸ **release** builds macOS, Windows, Linux and web in parallel (about 15–25 minutes),
   then creates the draft **PixelCraft v0.1.2-beta** (tag `v0.1.2-beta`) with every artifact,
   `SHA256SUMS.txt` and generated "What's Changed" notes.
5. On GitHub ▸ Releases, open the draft, check the notes and assets, then **Publish release**.
   Publishing creates the tag.

Pushing to `release` again before publishing refreshes the same draft. A published version is
never rebuilt: the workflow fails until the version is bumped. The workflow also runs from
Actions ▸ release ▸ **Run workflow**.

## Artifacts

| Platform | File | Built by |
|---|---|---|
| macOS 11+ (Apple silicon and Intel) | `pixelcraft-<v>-macos-universal.dmg` | `packaging/macos/package.sh --arch universal` |
| macOS CLI | `pixelcraft-cli-<v>-macos-universal.zip` | same |
| Windows x64 / ARM64 | `pixelcraft-<v>-windows-<arch>-portable.zip` (app + CLI) | `packaging/windows/package.ps1 -Arch x64\|arm64` |
| Linux x86_64 / aarch64 | `.AppImage` (+ `.zsync`), `.deb`, `.rpm`, `.tar.gz` | `packaging/linux/package.sh` (Ubuntu 22.04, glibc 2.35+) |
| Web | `pixelcraft-web-<v>.zip` (static site, `HOSTING.md` inside) | `packaging/web/package.sh` (trunk) |
| All | `SHA256SUMS.txt` | the `draft` job |

Every script writes to `dist/release/` and can be run locally, e.g.
`packaging/macos/package.sh --arch aarch64` on a Mac (needs only Xcode's command-line tools).

## Signing

| Platform | Today | With secrets |
|---|---|---|
| macOS | ad-hoc signed: on another Mac, right-click ▸ Open the first time (or `xattr -dr com.apple.quarantine /Applications/PixelCraft.app`) | `APPLE_CERTIFICATE` (base64 .p12, Developer ID Application), `APPLE_CERTIFICATE_PASSWORD`; notarized with `APPLE_ID`, `APPLE_PASSWORD` (app-specific), `APPLE_TEAM_ID` |
| Windows | unsigned: SmartScreen ▸ More info ▸ Run anyway | not wired yet |
| Linux | — | — |

Add the secrets under the repository's Settings ▸ Secrets and variables ▸ Actions; the workflow
picks them up without changes.

## Not yet

- Windows MSI installer (WiX) and code signing, as in the sibling Craft apps.
- File associations (`.aseprite`, `.ase`): added when PixelCraft can open them (`docs/gaps.md`).
- Flatpak.
- Switching to the shared `storytold/craft-common-actions` release workflow (the packaging
  scripts already follow its contract; its macOS job needs a larger paid runner).
