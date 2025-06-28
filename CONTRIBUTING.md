# Contributing to Error Report Web (Rust)

Thank you for your interest in contributing to the Yocto Project Error Reporting Web Application Rust implementation! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.75 or later
- PostgreSQL 12 or later (for development)
- Docker and Docker Compose (optional, for containerized development)
- Git

### Development Setup

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/moto-timo/error-report-web-rs.git
   cd error-report-web-rs
   ```

2. **Set up your development environment**:
   ```bash
   # Copy environment file
   cp .env.example .env
   # Edit .env with your local settings
   
   # Install Rust if you haven't already
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install additional components
   rustup component add rustfmt clippy
   ```

3. **Set up the database**:
   ```bash
   # Using Docker (recommended)
   docker-compose up -d db
   
   # Or install PostgreSQL locally and create database
   createdb error_reports_dev
   psql error_reports_dev < migrations/001_initial.sql
   ```

4. **Run the application**:
   ```bash
   cargo run
   ```

5. **Verify everything works**:
   ```bash
   # Run tests
   cargo test
   
   # Test error submission
   python3 test-data/test-send-error.py http://localhost:8000/ClientPost/JSON/ test-data/test-payload.json
   ```

## 🛠 Development Workflow

### Code Style

We follow standard Rust conventions:

- **Formatting**: Use `cargo fmt` to format your code
- **Linting**: Run `cargo clippy` and fix all warnings
- **Documentation**: Add documentation for public APIs
- **Testing**: Write tests for new features

```bash
# Before committing, always run:
cargo fmt
cargo clippy
cargo test
```

### Commit Guidelines

- Use clear, descriptive commit messages
- Follow the [Conventional Commits](https://www.conventionalcommits.org/) format when possible
- Keep commits atomic (one logical change per commit)

Examples:
```
feat: add search functionality to error list
fix: resolve database connection leak in stats service
docs: update API documentation for error submission
test: add integration tests for admin dashboard
```

### Branch Strategy

- `main` - Stable, production-ready code
- `develop` - Integration branch for new features
- `feature/description` - Feature development branches
- `fix/description` - Bug fix branches

## 📝 Making Changes

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Write clean, well-documented code
- Add tests for new functionality
- Update documentation as needed
- Follow existing code patterns and conventions

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Test specific modules
cargo test handlers::api

# Run integration tests
cargo test --test integration_tests

# Manual testing with the test script
python3 test-data/test-send-error.py http://localhost:8000/ClientPost/JSON/ test-data/test-payload.json
```

### 4. Submit a Pull Request

1. Push your branch to your fork
2. Create a Pull Request against the `main` branch
3. Fill out the PR template with:
   - Description of changes
   - Testing performed
   - Breaking changes (if any)
   - Screenshots (for UI changes)

## 🏗 Project Structure

```
error-report-web-rs/
├── src/
│   ├── handlers/          # HTTP request handlers
│   ├── models/           # Database models and DTOs
│   ├── services/         # Business logic services
│   ├── utils/           # Utility functions
│   ├── config.rs        # Configuration management
│   ├── lib.rs          # Library exports
│   └── main.rs         # Application entry point
├── templates/           # HTML templates
├── static/             # Static assets (CSS, JS, images)
├── migrations/         # Database migrations
├── test-data/          # Test data and scripts
└── tests/              # Integration tests
```

## 🧪 Testing

### Unit Tests

Write unit tests in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_function() {
        // Test implementation
    }
}
```

### Integration Tests

Add integration tests in the `tests/` directory:

```rust
#[tokio::test]
async fn test_api_endpoint() {
    // Test implementation
}
```

### Manual Testing

Use the provided test script to verify API functionality:

```bash
python3 test-data/test-send-error.py http://localhost:8000/ClientPost/JSON/ test-data/test-payload.json
```

## 📚 Documentation

### Code Documentation

- Add rustdoc comments for public APIs
- Include examples in documentation when helpful
- Keep documentation up-to-date with code changes

```rust
/// Submits a new error report to the database
/// 
/// # Arguments
/// 
/// * `payload` - The error report data to submit
/// 
/// # Returns
/// 
/// Returns the saved error report with assigned ID
/// 
/// # Example
/// 
/// ```rust
/// let response = submit_error_report(payload).await?;
/// println!("Error report saved with ID: {}", response.id);
/// ```
pub async fn submit_error_report(payload: ErrorSubmissionData) -> Result<SubmissionResponse, Error> {
    // Implementation
}
```

### README Updates

Update the README.md when adding:
- New features
- Configuration options
- Dependencies
- Setup instructions

## 🐛 Reporting Issues

When reporting bugs, please include:

1. **Environment details**:
   - Operating system
   - Rust version (`rustc --version`)
   - Database version

2. **Steps to reproduce**:
   - Detailed step-by-step instructions
   - Sample data or configuration

3. **Expected vs actual behavior**:
   - What you expected to happen
   - What actually happened

4. **Logs and error messages**:
   - Relevant log output
   - Full error messages and stack traces

## 🔒 Security

### Reporting Security Issues

Please report security vulnerabilities privately to:
- Email: ticotimo@gmail.com
- Do not create public issues for security vulnerabilities

### Security Best Practices

- Validate all input data
- Use parameterized queries for database operations
- Implement proper authentication and authorization
- Keep dependencies up-to-date
- Follow OWASP guidelines

## 🌟 Feature Requests

When requesting features:

1. **Check existing issues** to avoid duplicates
2. **Describe the use case** clearly
3. **Explain the benefit** to users
4. **Consider implementation complexity**
5. **Provide examples** of desired behavior

## 📋 Code Review Guidelines

### For Reviewers

- Be constructive and respectful
- Focus on code quality, security, and maintainability
- Test the changes locally when possible
- Provide specific, actionable feedback

### For Contributors

- Respond to feedback promptly and professionally
- Make requested changes in additional commits
- Ask questions if feedback is unclear
- Update the PR description if scope changes

## 🎯 Performance Considerations

When contributing, consider:

- **Database query efficiency**: Use indexes appropriately
- **Memory usage**: Avoid unnecessary allocations
- **Error handling**: Use proper Result types
- **Async patterns**: Use async/await correctly
- **Caching**: Consider caching for frequently accessed data

## 📖 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Yocto Project Documentation](https://docs.yoctoproject.org/)
- [Original Django Implementation](https://git.yoctoproject.org/error-report-web/)

## 💬 Community

- GitHub Discussions: [https://github.com/moto-timo/error-report-web-rs/discussions](https://github.com/moto-timo/error-report-web-rs/discussions)
- Issues: [https://github.com/moto-timo/error-report-web-rs/issues](https://github.com/moto-timo/error-report-web-rs/issues)

Thank you for contributing to the Yocto Project Error Reporting System! 🦀
