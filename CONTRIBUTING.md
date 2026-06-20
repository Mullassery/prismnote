# Contributing to PrismNote

Welcome! We're excited you want to contribute to PrismNote. This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to the Contributor Covenant Code of Conduct. All contributors must be respectful and inclusive.

## Ways to Contribute

### 1. Report Bugs
Found a bug? Please [open an issue](https://github.com/Mullassery/prismnote/issues) with:
- Clear description of the problem
- Steps to reproduce
- Expected behavior vs actual behavior
- Screenshots (if applicable)
- System info (OS, Python version, Rust version)

### 2. Suggest Features
Have an idea? [Open a discussion](https://github.com/Mullassery/prismnote/discussions) or issue with:
- Clear description of the feature
- Why it's needed (use case)
- How it would work (proposed implementation)
- Any alternatives you've considered

### 3. Submit Pull Requests
See [Pull Request Process](#pull-request-process) below.

### 4. Improve Documentation
- Fix typos in docs
- Clarify confusing sections
- Add examples
- Translate docs to other languages

### 5. Help with Testing
- Test PRs and report issues
- Test on different platforms
- Performance testing
- Edge case testing

## Development Setup

### Prerequisites
- Rust 1.70+
- Python 3.8+
- Node.js 18+
- Git

### Local Development

```bash
# Clone the repo
git clone https://github.com/Mullassery/prismnote.git
cd prismnote

# Setup git hooks (recommended)
git config core.hooksPath .git/hooks

# Install Python kernel support
pip install ipykernel

# Build everything
bash build.sh

# Run development servers
# Terminal 1: Rust backend
cargo run --release

# Terminal 2: React frontend (in another terminal)
cd frontend && npm run dev
```

Then visit:
- Frontend: http://localhost:5173
- Backend: http://localhost:8000

## Project Structure

```
prismnote/
 crates/server/              # Rust backend
    src/
       main.rs            # Entry point
       api.rs             # REST endpoints
       kernel.rs          # Python execution
       ai.rs              # AI integration
       db.rs              # Database connectors
       ...                # Other modules
    Cargo.toml
 frontend/                   # React + TypeScript
    src/
       components/        # React components
       hooks/             # Custom hooks
       styles/            # CSS/Tailwind
    package.json
 python/                     # pip/uv package
 docs/                       # Documentation
```

## Coding Standards

### Rust
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write doc comments for public items
- Keep functions small and focused
- Prefer `Result<T>` over `unwrap()` for error handling

### React/TypeScript
- Use TypeScript (no `any` types)
- Functional components with hooks
- Use Tailwind CSS for styling
- Keep components under 300 lines
- Export types from files

### Git Commits
- Use clear, descriptive commit messages
- Start with a verb: "Add", "Fix", "Update", "Remove"
- Reference issues: "Fixes #123"
- Keep commits focused (one feature per commit)

Example:
```
Fix kernel execution hanging on large output

- Added output buffering for stdout/stderr
- Limit initial output to 10K chars
- Add "show more" button for full output
- Fixes #456
```

## Pull Request Process

### Before You Start
1. Check if an issue exists for what you're working on
2. Fork the repo
3. Create a branch: `git checkout -b feature/your-feature`
4. Keep your branch up to date with main

### Making Changes
1. Make your changes
2. Test locally
3. Add/update tests if applicable
4. Update documentation
5. Run linters and formatters:
   ```bash
   # Rust
   cargo fmt
   cargo clippy
   
   # TypeScript
   npm run lint
   ```

### Before Submitting
1. Ensure tests pass: `cargo test` and `npm run build`
2. Update CHANGELOG.md
3. Squash related commits (keep history clean)
4. Write a clear PR description

### PR Description Template

```markdown
## Description
Brief description of what this PR does.

## Related Issues
Fixes #123
Related to #456

## Changes
- Change 1
- Change 2
- Change 3

## Testing
- [ ] Manual testing on macOS
- [ ] Manual testing on Linux
- [ ] Manual testing on Windows
- [ ] All existing tests pass
- [ ] New tests added (if applicable)

## Performance Impact
- Memory: No change / +5% / etc
- Speed: No change / -10ms per cell / etc

## Screenshots (if UI changes)
[Add screenshots showing before/after]

## Checklist
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] No breaking changes
- [ ] All tests pass
- [ ] Commit messages are clear
```

## Implementation Notes

### Before Building Features
1. Check `ARCHITECTURE.md` to see if the feature is:
   - ✅ Already implemented (ready to use)
   - 🟡 Framework-only (has code structure but not integrated)
   - ❌ Not started (needs full implementation)

2. For framework-only features, you need to:
   - Complete the backend implementation
   - Create/connect frontend components
   - Wire API endpoints to handlers
   - Add end-to-end testing

3. See `ARCHITECTURE.md` for detailed feature maturity status

## Testing

### Running Tests
```bash
# Rust tests
cargo test

# Frontend tests (when ready)
npm run test

# Integration tests
cargo test --release
```

### Writing Tests
- Place unit tests in the same file as the code
- Place integration tests in `tests/` directory
- Aim for >70% code coverage on new code
- Test happy path AND error cases
- For framework-only features, test that code at least compiles

## Documentation

### Updating Docs
- Keep README.md up to date
- Document new features in the appropriate guide
- Update feature comparison if applicable
- Add code examples for complex features
- Include links to related issues

### Documentation Files
- `README.md` - Project overview and quick start
- `ARCHITECTURE.md` - Implementation status (what's built vs. framework-only)
- `SQL_EXECUTION.md` - SQL cell usage and setup
- `CLOUD_WAREHOUSES.md` - Cloud data warehouse integration
- `SPARK_MANAGEMENT.md` - Spark configuration
- `ENTERPRISE_AUTHENTICATION.md` - Auth provider setup
- `AI_TRAINING_FINETUNING.md` - Model fine-tuning with RunPod
- See `docs/archive/` for older planning documents

## Release Process

Maintainers release updates following semantic versioning:

- **Patch** (0.1.1): Bug fixes, documentation
- **Minor** (0.2.0): New features, backwards compatible
- **Major** (1.0.0): Breaking changes, major refactor

Releases are tagged and published to:
- GitHub Releases
- PyPI (for Python package)

## Getting Help

- **GitHub Issues**: Report bugs and request features
- **GitHub Discussions**: Ask questions, share ideas
- **Discord** (coming soon): Real-time chat with maintainers

## Recognition

Contributors will be:
- Added to CONTRIBUTORS.md
- Mentioned in release notes
- Listed in GitHub contributors graph

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Open a [GitHub Discussion](https://github.com/Mullassery/prismnote/discussions) or ask in an issue. We're here to help!

---

**Thank you for contributing to PrismNote!** 
