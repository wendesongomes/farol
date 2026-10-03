## What this PR does

<!-- One or two sentences. Link the issue it closes, if any: "Closes #123". -->

## How it was tested

<!-- Commands you ran and, for platform-specific changes, the system you tested on. -->

- [ ] `cargo fmt --check`, `cargo clippy` and `cargo test` (in `src-tauri`)
- [ ] `npm run lint` and `npm test`
- [ ] Tested manually on: <!-- macOS / Windows / Linux (+ desktop environment) -->

## Checklist

- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
- [ ] New detection rules have a case in `rules/detection_tests.toml`
- [ ] New user-facing text is in `src/i18n/en.ts` and `src/i18n/pt-BR.ts`
- [ ] No new network access other than the `localhost` probe
