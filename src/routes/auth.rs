use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api_state::AppState;
use crate::security::{AuthService, Claims, AuditService, extract_claims};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize, FromRow, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    pub user: UserResponse,
    pub message: String,
}

/// Login endpoint
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    req: web::Json<LoginRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let auth_service = AuthService::new().unwrap();
    let audit_service = AuditService::new(data.db_pool.clone());

    // Find user
    let user_result: Result<Option<(Uuid, String, String, Option<String>, String, bool)>, sqlx::Error> = sqlx::query_as(
        "SELECT id, username, email, full_name, password_hash, active FROM users WHERE username = $1"
    )
    .bind(&req.username)
    .fetch_optional(&data.db_pool)
    .await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            audit_service.log_login(None, &req.username, None, None, false, Some("User not found")).await.ok();
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    let (user_id, username, email, full_name, password_hash, active) = user;

    if !active {
        audit_service.log_login(Some(user_id), &username, None, None, false, Some("Account inactive")).await.ok();
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Account is inactive"
        }));
    }

    // Verify password
    if !verify(&req.password, &password_hash).unwrap_or(false) {
        audit_service.log_login(Some(user_id), &username, None, None, false, Some("Invalid password")).await.ok();
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        }));
    }

    // Get user roles
    let roles: Vec<String> = sqlx::query_scalar(
        "SELECT r.name FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = $1"
    )
    .bind(&user_id)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    // Update last login
    sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
        .bind(&Utc::now())
        .bind(&user_id)
        .execute(&data.db_pool)
        .await
        .ok();

    // Generate token
    let claims = Claims::new(user_id.to_string(), username.clone(), roles.clone());
    let token = match auth_service.generate_token(&claims) {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to generate token: {}", e)
            }));
        }
    };

    audit_service.log_login(Some(user_id), &username, None, None, true, None).await.ok();

    HttpResponse::Ok().json(LoginResponse {
        token,
        user: UserResponse {
            id: user_id,
            username,
            email,
            full_name,
            roles,
        },
    })
}

/// Register endpoint (admin only)
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created", body = RegisterResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Forbidden - admin only")
    )
)]
pub async fn register(
    req: web::Json<RegisterRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    // Hash password
    let password_hash = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to hash password: {}", e)
            }));
        }
    };

    // Create user
    let user_id = Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, full_name) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&user_id)
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.full_name)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            // Assign default viewer role
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) 
                 SELECT $1, id FROM roles WHERE name = 'viewer'"
            )
            .bind(&user_id)
            .execute(&data.db_pool)
            .await
            .ok();

            HttpResponse::Created().json(RegisterResponse {
                user: UserResponse {
                    id: user_id,
                    username: req.username.clone(),
                    email: req.email.clone(),
                    full_name: req.full_name.clone(),
                    roles: vec!["viewer".to_string()],
                },
                message: "User created successfully".to_string(),
            })
        }
        Err(sqlx::Error::Database(e)) if e.constraint().is_some() => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Username or email already exists"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

/// Get current user info
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "Current user info", body = UserResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_me(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let auth_service = AuthService::new().unwrap();
    
    let claims = match extract_claims(&req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Fetch full user data from database
    let user_result: Result<Option<(Uuid, String, String, Option<String>)>, sqlx::Error> = sqlx::query_as(
        "SELECT id, username, email, full_name FROM users WHERE id = $1"
    )
    .bind(&Uuid::parse_str(&claims.sub).unwrap())
    .fetch_optional(&data.db_pool)
    .await;

    match user_result {
        Ok(Some((id, username, email, full_name))) => {
            // Get roles
            let roles: Vec<String> = sqlx::query_scalar(
                "SELECT r.name FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = $1"
            )
            .bind(&id)
            .fetch_all(&data.db_pool)
            .await
            .unwrap_or_default();

            HttpResponse::Ok().json(UserResponse {
                id,
                username,
                email,
                full_name,
                roles,
            })
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        }))
    }
}
