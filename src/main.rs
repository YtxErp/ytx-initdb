mod constant;
mod database;
mod encrypt;
mod schema;

use crate::database::*;
use crate::encrypt::encrypt_password;
use crate::encrypt::hex_to_bytes;

use dotenvy::dotenv;
use std::env::var;

fn main() -> Result<(), String> {
    // Load environment variables
    dotenv().ok();

    // Read default data from .env file
    let postgres_url = var("POSTGRES_URL").expect("POSTGRES_URL must be set");
    let auth_db = var("AUTH_DB").expect("AUTH_DB must be set");
    let main_db = var("MAIN_DB").expect("MAIN_DB must be set");
    let main_workspace = var("MAIN_WORKSPACE").expect("MAIN_WORKSPACE must be set");
    let enc_key_str = var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set");

    let enc_key = hex_to_bytes(&enc_key_str)
        .map_err(|e| format!("Failed to hex_to_bytes '{}': {}", enc_key_str, e))?;

    println!("{:?}", enc_key);

    validate_name(&auth_db)?;
    validate_name(&main_db)?;

    // Read default users from .env file
    let admin_user = var("ADMIN_USER").expect("ADMIN_USER must be set");
    validate_name(&admin_user)?;
    let admin_password = var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set");

    let readonly_user = var("READONLY_USER").expect("READONLY_USER must be set");
    validate_name(&readonly_user)?;
    let readonly_password = var("READONLY_PASSWORD").expect("READONLY_PASSWORD must be set");

    let readwrite_user = var("READWRITE_USER").expect("READWRITE_USER must be set");
    validate_name(&readwrite_user)?;
    let readwrite_password = var("READWRITE_PASSWORD").expect("READWRITE_PASSWORD must be set");

    let mut postgres_client =
        postgres::Client::connect(&postgres_url, postgres::NoTls).map_err(|e| {
            format!(
                "Failed to connect to PostgreSQL with url '{}': {}",
                postgres_url, e
            )
        })?;

    // 1. Create databases
    create_database(&mut postgres_client, &auth_db)
        .map_err(|e| format!("Failed to create auth database '{}': {}", auth_db, e))?;

    create_database(&mut postgres_client, &main_db)
        .map_err(|e| format!("Failed to create main database '{}': {}", main_db, e))?;

    // 2. Create users (roles)
    create_user(&mut postgres_client, &admin_user, &admin_password)
        .map_err(|e| format!("Failed to create admin user '{}': {}", admin_user, e))?;

    create_user(&mut postgres_client, &readonly_user, &readonly_password)
        .map_err(|e| format!("Failed to create readonly user '{}': {}", readonly_user, e))?;

    create_user(&mut postgres_client, &readwrite_user, &readwrite_password).map_err(|e| {
        format!(
            "Failed to create readwrite user '{}': {}",
            readwrite_user, e
        )
    })?;

    // Replace the database name in the Postgres URL for the auth database
    let auth_url = replace_postgres_url(&postgres_url, &auth_db).map_err(|e| {
        format!(
            "Failed to replace DB in URL for auth DB '{}': {}",
            auth_db, e
        )
    })?;

    // Connect to the auth database
    let mut auth_client = postgres::Client::connect(&auth_url, postgres::NoTls)
        .map_err(|e| format!("Failed to connect to auth DB at '{}': {}", auth_url, e))?;

    // Initialize the auth database schema (create tables, etc.)
    initialize_auth_database(&mut auth_client)
        .map_err(|e| format!("Failed to initialize auth database '{}': {}", auth_db, e))?;

    // Insert the workspace and database mapping into auth database
    insert_workspace_database(&mut auth_client, &main_workspace, &main_db).map_err(|e| {
        format!(
            "Failed to insert workspace database link (workspace='{}', db='{}'): {}",
            main_workspace, main_db, e
        )
    })?;

    // Encrypt readonly user password and insert corresponding role record into auth database
    let (readonly_enc_password, readonly_nonce) = encrypt_password(&readonly_password, &enc_key)
        .map_err(|e| format!("Failed to encrypt readonly user password {}", e))?;

    insert_database_role(
        &mut auth_client,
        &auth_db,
        &readonly_user,
        &readonly_enc_password,
        &readonly_nonce,
    )
    .map_err(|e| {
        format!(
            "Failed to insert readonly role for '{}': {}",
            readonly_user, e
        )
    })?;

    // Encrypt readwrite user password and insert corresponding role record into auth database
    let (readwrite_enc_password, readwrite_nonce) = encrypt_password(&readwrite_password, &enc_key)
        .map_err(|e| format!("Failed to encrypt readwrite user password {}", e))?;
    insert_database_role(
        &mut auth_client,
        &auth_db,
        &readwrite_user,
        &readwrite_enc_password,
        &readwrite_nonce,
    )
    .map_err(|e| {
        format!(
            "Failed to insert readwrite role for '{}': {}",
            readwrite_user, e
        )
    })?;

    let main_url = replace_postgres_url(&postgres_url, &main_db).map_err(|e| {
        format!(
            "Failed to replace DB in URL for main DB '{}': {}",
            main_db, e
        )
    })?;

    let mut main_client = postgres::Client::connect(&main_url, postgres::NoTls)
        .map_err(|e| format!("Failed to connect to main DB at '{}': {}", main_url, e))?;

    initialize_main_database(&mut main_client)
        .map_err(|e| format!("Failed to initialize main database '{}': {}", main_db, e))?;

    grant_readonly_permission(
        &mut postgres_client,
        &mut main_client,
        &main_db,
        &readonly_user,
    )
    .map_err(|e| {
        format!(
            "Failed to grant readonly permission to user '{}': {}",
            readonly_user, e
        )
    })?;

    grant_readwrite_permission(
        &mut postgres_client,
        &mut main_client,
        &main_db,
        &readwrite_user,
    )
    .map_err(|e| {
        format!(
            "Failed to grant readwrite permission to user '{}': {}",
            readwrite_user, e
        )
    })?;

    grant_admin_permission(
        &mut postgres_client,
        &mut main_client,
        &main_db,
        &admin_user,
    )
    .map_err(|e| {
        format!(
            "Failed to grant admin_user permission to user '{}': {}",
            admin_user, e
        )
    })?;

    grant_admin_permission(
        &mut postgres_client,
        &mut auth_client,
        &auth_db,
        &admin_user,
    )
    .map_err(|e| {
        format!(
            "Failed to grant admin_user permission to user '{}': {}",
            admin_user, e
        )
    })?;

    grant_auth_read_permission(&mut postgres_client, &mut auth_client, &readonly_user).map_err(
        |e| {
            format!(
                "Failed to grant read permission to readonly user '{}': {}",
                readonly_user, e
            )
        },
    )?;

    grant_auth_read_permission(&mut postgres_client, &mut auth_client, &readwrite_user).map_err(
        |e| {
            format!(
                "Failed to grant read permission to readwrite user '{}': {}",
                readwrite_user, e
            )
        },
    )?;
    Ok(())
}
