# Rausth

A reusable authentication server API in Rust using Rocket framework.

## Features

- **Multiple Authentication Methods**:
  - Email + Password authentication
  - Google OAuth
  - Microsoft OAuth  
  - GitHub OAuth

- **Token-based Authentication**:
  - JWT access tokens
  - Refresh tokens with database storage
  - Configurable token expiration

- **User Management**:
  - Automatic user creation on first authentication
  - User data storage in PostgreSQL

- **Flexible Configuration**:
  - Enable/disable authentication methods via configuration
  - Support for both TOML config files and environment variables

## Quick Start

```bash
# Clone the repository
git clone https://github.com/Souchy/Rausth.git
cd Rausth

# Set up database
createdb rausth

# Configure (copy and edit)
cp .env.example .env

# Build and run
cargo build --release
cargo run --release
```

The server will start on `http://localhost:8000`.

## Documentation

- [Usage Guide](USAGE.md) - Detailed API usage examples
- [Contributing](CONTRIBUTING.md) - How to contribute to the project

## Requirements

- Rust 1.70 or higher
- PostgreSQL database

## Installation

1. Clone the repository:
```bash
git clone https://github.com/Souchy/Rausth.git
cd Rausth
```

2. Set up PostgreSQL database:
```bash
createdb rausth
```

3. Configure the application (choose one method):

   **Option A: Using environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

   **Option B: Using config file**
   ```bash
   cp config.toml.example config.toml
   # Edit config.toml with your configuration
   ```

4. Build and run:
```bash
cargo build --release
cargo run --release
```

## Configuration

### Required Settings

- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key for JWT token signing

### Authentication Methods

Enable or disable authentication methods in your configuration:

- `AUTH_EMAIL_PASSWORD`: Enable email/password authentication (default: true)
- `AUTH_GOOGLE_ENABLED`: Enable Google OAuth (default: false)
- `AUTH_MICROSOFT_ENABLED`: Enable Microsoft OAuth (default: false)
- `AUTH_GITHUB_ENABLED`: Enable GitHub OAuth (default: false)

### OAuth Provider Configuration

For each OAuth provider you want to enable:

1. Register your application with the provider
2. Obtain client ID and client secret
3. Configure the redirect URI
4. Add the credentials to your configuration

See `.env.example` or `config.toml.example` for detailed configuration options.

## API Endpoints

### Email/Password Authentication

**Register a new user**
```http
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Login**
```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword"
}
```

### OAuth Authentication

**Google OAuth**
```http
GET /auth/oauth/google
```
Redirects to Google login page.

```http
POST /auth/oauth/google/callback
Content-Type: application/json

{
  "code": "authorization_code_from_google"
}
```

**Microsoft OAuth**
```http
GET /auth/oauth/microsoft
```

```http
POST /auth/oauth/microsoft/callback
Content-Type: application/json

{
  "code": "authorization_code_from_microsoft"
}
```

**GitHub OAuth**
```http
GET /auth/oauth/github
```

```http
POST /auth/oauth/github/callback
Content-Type: application/json

{
  "code": "authorization_code_from_github"
}
```

### Token Refresh

```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "your_refresh_token"
}
```

## Response Format

All authentication endpoints return the same response format:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "550e8400-e29b-41d4-a716-446655440000",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

## Database Schema

The application automatically creates the required database tables on startup:

- `users`: Stores user information and authentication details
- `refresh_tokens`: Stores refresh tokens with expiration

## Security Considerations

- Always use HTTPS in production
- Change the default JWT secret
- Use strong passwords for database connections
- Rotate refresh tokens after use
- Set appropriate token expiration times
- Keep OAuth client secrets secure

## Development

Build the project:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

Run in development mode:
```bash
cargo run
```

Format code:
```bash
cargo fmt
```

Check for issues:
```bash
cargo clippy
```

## Examples

See [USAGE.md](USAGE.md) for detailed usage examples including:
- Complete authentication flows
- OAuth provider setup
- Integration with frontend applications
- Error handling

## License

This project is available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Roadmap

Future enhancements:
- Additional OAuth providers (Apple, Twitter, etc.)
- Two-factor authentication (2FA)
- Email verification
- Password reset functionality
- Rate limiting
- Session management
- Admin API for user management

