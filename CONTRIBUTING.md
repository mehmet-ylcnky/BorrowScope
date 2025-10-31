# Contributing to BorrowScope

Thank you for your interest in contributing to BorrowScope!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/BorrowScope.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Format code: `cargo fmt`
7. Run clippy: `cargo clippy`
8. Commit: `git commit -m "Add your feature"`
9. Push: `git push origin feature/your-feature-name`
10. Open a Pull Request

## Code Standards

### Formatting

We use `rustfmt` for consistent code formatting:

```bash
cargo fmt
```

### Linting

All code must pass `clippy` without warnings:

```bash
cargo clippy -- -D warnings
```

### Testing

Add tests for new features:

```bash
cargo test
```

### Commit Messages

Use clear, descriptive commit messages:

- ✅ "Add borrow tracking for RefCell"
- ✅ "Fix lifetime inference in macro expansion"
- ❌ "fix stuff"
- ❌ "wip"

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add entry to CHANGELOG.md (if applicable)
4. Request review from maintainers
5. Address review feedback
6. Maintainer will merge once approved

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

## Questions?

Open an issue or start a discussion on GitHub.

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
