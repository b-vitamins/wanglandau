# AGENT DEVELOPMENT GUIDELINES

This repository contains a Rust crate implementing the Wang--Landau algorithm. All automated agents should follow these standards when contributing.

## Commit Message Standards
- Use **Conventional Commits** (https://www.conventionalcommits.org/) for all changes.
- Begin the subject line with one of `feat:`, `fix:`, `docs:`, `chore:`, `test:`, or `refactor:`.
- Keep messages concise; describe _why_ the change was made when not obvious.

## Pull Request Guidelines
- Title should summarize the change using a conventional commit style prefix.
- Include a summary of changes, testing performed, and any relevant issues.
- Ensure `cargo test` passes before requesting review.

## Development Workflow
- Group related changes into atomic commits.
- Run `cargo fmt` and `cargo clippy` before committing, if available.
- Update `CHANGELOG.md` for user facing changes.
- Keep the `master` branch clean; open PRs from topic branches when working manually.

## Versioning
- This project follows [Semantic Versioning](https://semver.org/).
- Increment the PATCH version for fixes, MINOR for new backwards compatible features, and MAJOR for breaking changes.

## Testing Standards
- Tests are written with Rust's built in test framework.
- Maintain high coverage for core functionality.
- New features should include appropriate unit tests in the `tests/` directory.

## Pre-commit Checks
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`

Agents should ensure these commands succeed before committing changes.
