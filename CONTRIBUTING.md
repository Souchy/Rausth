# Contributing to Rausth

Thank you for your interest in contributing to Rausth! This document provides guidelines and information for contributors.

## Development Setup

1. **Clone the repository**:
```bash
git clone https://github.com/Souchy/Rausth.git
cd Rausth
```

2. **Install Rust** (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

3. **Install PostgreSQL** and create a test database:
```bash
createdb rausth_test
```

4. **Set up environment variables**:
```bash
cp .env.example .env
# Edit .env with your local configuration
```

5. **Build the project**:
```bash
cargo build
```

6. **Run the project**:
```bash
cargo run
```

## Code Style

This project follows the standard Rust style guidelines:

- Use `cargo fmt` to format code
- Use `cargo clippy` to check for common mistakes
- Follow Rust naming conventions

## Testing

Currently, the project doesn't have automated tests. Contributors are welcome to add:
- Unit tests for authentication logic
- Integration tests for API endpoints
- End-to-end tests for OAuth flows

## Adding New Features

When adding new features:

1. **Create a new branch** for your feature:
```bash
git checkout -b feature/your-feature-name
```

2. **Make your changes** following the existing code structure

3. **Test your changes** thoroughly

4. **Document your changes** in code comments and README if needed

5. **Submit a pull request** with:
   - Clear description of what the feature does
   - Why it's needed
   - Any breaking changes

## Project Structure

```
src/
├── auth/           # Authentication implementations
│   ├── email.rs    # Email/password authentication
│   ├── jwt.rs      # JWT token generation and verification
│   ├── oauth.rs    # OAuth provider implementations
│   └── refresh.rs  # Refresh token handling
├── config/         # Configuration management
├── db/             # Database connections and migrations
├── models/         # Data models and request/response types
├── routes/         # API endpoint handlers
├── utils/          # Utility functions and error handling
└── main.rs         # Application entry point
```

## Adding a New OAuth Provider

To add support for a new OAuth provider:

1. **Update `OAuthProvider` enum** in `src/auth/oauth.rs`:
```rust
pub enum OAuthProvider {
    Google,
    Microsoft,
    GitHub,
    NewProvider,  // Add your provider
}
```

2. **Add provider configuration** in `src/config/mod.rs`:
```rust
pub struct AuthConfig {
    // ... existing providers
    pub new_provider: Option<OAuthProviderConfig>,
}
```

3. **Implement user info retrieval** in `src/auth/oauth.rs`:
```rust
async fn get_new_provider_user_info(&self, access_token: &str) -> AuthResult<(String, String)> {
    // Implementation here
}
```

4. **Add routes** in `src/routes/oauth.rs`

5. **Update documentation** with the new provider setup instructions

## Bug Reports

When reporting bugs, please include:

- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected behavior
- Actual behavior
- Any error messages or logs

## Feature Requests

Feature requests are welcome! Please:

- Check if the feature has already been requested
- Explain the use case
- Describe the expected behavior
- Consider if it fits the project's scope

## Code Review

All submissions require review. We use GitHub pull requests for this:

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to your fork
5. Submit a pull request

## License

By contributing to Rausth, you agree that your contributions will be licensed under the project's MIT License.

## Questions?

Feel free to open an issue for questions or discussions about the project.
