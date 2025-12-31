use crate::models::{AuthResponse, LoginRequest, RegisterRequest};
use crate::utils::{AuthError, AuthResult};
use crate::auth::jwt::JwtService;
use crate::db::DatabaseService;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use uuid::Uuid;

/// Email/Password authentication service.
///
/// This service handles user registration and login with email and password.
/// Passwords are hashed using bcrypt before storage.
pub struct EmailPasswordAuth {
    jwt_service: JwtService,
}

impl EmailPasswordAuth {
    /// Create a new email/password authentication service.
    ///
    /// # Arguments
    /// * `jwt_service` - JWT service for generating application tokens
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }

    /// Register a new user with email and password.
    ///
    /// This method creates a new user account, hashing the password with bcrypt.
    /// Returns an error if the email is already registered.
    ///
    /// # Arguments
    /// * `db` - Database service for user management
    /// * `request` - Registration request containing email and password
    ///
    /// # Returns
    /// Authentication response with JWT tokens
    pub async fn register(
        &self,
        db: &dyn DatabaseService,
        request: RegisterRequest,
    ) -> AuthResult<AuthResponse> {
        // Check if user already exists
        let existing_user = db.find_user_by_email_and_provider(&request.email, "email").await?;

        if existing_user.is_some() {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash the password
        let password_hash = hash(&request.password, DEFAULT_COST)?;

        // Create the user
        let user = db.create_user(&request.email, Some(&password_hash), "email", None).await?;

        // Generate tokens
        self.generate_auth_response(db, user.id).await
    }

    /// Authenticate a user with email and password.
    ///
    /// This method verifies the user's credentials and generates new JWT tokens.
    ///
    /// # Arguments
    /// * `db` - Database service for user management
    /// * `request` - Login request containing email and password
    ///
    /// # Returns
    /// Authentication response with JWT tokens
    pub async fn login(
        &self,
        db: &dyn DatabaseService,
        request: LoginRequest,
    ) -> AuthResult<AuthResponse> {
        // Fetch the user
        let user = db.find_user_by_email_and_provider(&request.email, "email")
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        let password_hash = user.password_hash.ok_or(AuthError::InvalidCredentials)?;
        let valid = verify(&request.password, &password_hash)?;

        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate tokens
        self.generate_auth_response(db, user.id).await
    }

    async fn generate_auth_response(
        &self,
        db: &dyn DatabaseService,
        user_id: Uuid,
    ) -> AuthResult<AuthResponse> {
        let access_token = self.jwt_service.generate_access_token(user_id)?;
        let refresh_token = self.jwt_service.generate_refresh_token();
        let expires_in = self.jwt_service.get_access_token_expiry_seconds();

        // Store refresh token in database
        let refresh_token_expiry = Utc::now() 
            + chrono::Duration::seconds(self.jwt_service.get_refresh_token_expiry_seconds());

        db.store_refresh_token(user_id, &refresh_token, refresh_token_expiry).await?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }
}
