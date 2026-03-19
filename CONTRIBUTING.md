# Contributing to KolibriOS AI

Thank you for your interest in contributing to KolibriOS AI! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/KolibriOS-AI.git`
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `make test`
6. Commit your changes: `git commit -m "Add your feature"`
7. Push to your fork: `git push origin feature/your-feature-name`
8. Create a Pull Request

## Development Setup

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- GCC or Clang (for C++ components)
- NASM (for assembly)
- QEMU (for testing)

### Building

```bash
# Install Rust targets
rustup target add x86_64-unknown-none
rustup component add rust-src

# Build all components
make all

# Run tests
make test

# Build documentation
make doc
```

## Project Structure

```
KolibriOS-AI/
├── kernel/           # Microkernel implementation
├── cells/            # Living Cell Architecture
├── koli_lang/        # Kolibri Language
├── unified_ai_agent/ # AI Agent System
├── apps/             # System Applications
├── docs/             # Documentation
└── .github/          # GitHub Actions
```

## Coding Standards

### Rust

- Use `cargo fmt` for formatting
- Run `cargo clippy` and fix all warnings
- Document public APIs with doc comments
- Write unit tests for new functionality

### C++

- Use clang-format with the project's .clang-format file
- Follow the C++20 standard
- Use RAII for resource management

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters
- Reference issues and pull requests liberally

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add tests for new functionality
4. Request review from maintainers
5. Address review feedback

## Areas for Contribution

- **Kernel Development**: Core microkernel implementation
- **Cell Architecture**: Building autonomous system cells
- **Koli Language**: Compiler and runtime development
- **AI Integration**: Unified AI agent development
- **Documentation**: Improving guides and references
- **Testing**: Writing tests and improving coverage
- **Drivers**: Hardware support

## Questions?

Open an issue for discussion or reach out to the maintainers.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
