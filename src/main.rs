mod constant;
mod database;
mod schema;

use crate::constant::*;
use crate::database::*;
use anyhow::{Context, Result, bail};
use dotenvy::dotenv;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde_json::Value;
use std::env::var;
use unicode_xid::UnicodeXID;

fn main() -> Result<()> {
    dotenv().ok();

    // Connection
    let postgres_url =
        var("POSTGRES_URL").unwrap_or_else(|_| "postgres://localhost:5432/postgres".to_string());
    let vault_addr = var("VAULT_URL").unwrap_or_else(|_| "http://127.0.0.1:8200".to_string());

    // Database names
    let auth_db = read_value_with_default("AUTH_DB", "ytx_auth")?;
    let main_db = read_value_with_default("MAIN_DB", "ytx_main")?;
    let main_workspace = read_workspace_with_default("MAIN_WORKSPACE", "ytx_workspace")?;

    // Roles
    let postgres_role = read_value_with_default("POSTGRES_ROLE", "postgres")?;

    // Passwords (Vault)
    let vault_token = var("VAULT_TOKEN")
        .context("VAULT_TOKEN is required to fetch database passwords from Vault")?;
    if vault_token.is_empty() {
        anyhow::bail!("VAULT_TOKEN is empty");
    }

    let pg_passwords = read_vault_data(&vault_addr, &vault_token, POSTGRES_SECRET_PATH)
        .context("Failed to read PostgreSQL superuser password from Vault")?;
    let postgres_password = get_vault_password(&pg_passwords, &postgres_role)?;

    let ytx_passwords = read_vault_data(&vault_addr, &vault_token, YTX_SECRET_PATH)
        .context("Failed to read YTX role passwords from Vault")?;
    let auth_readwrite_password = get_vault_password(&ytx_passwords, AUTH_READWRITE_ROLE)?;
    let main_readonly_password = get_vault_password(&ytx_passwords, MAIN_READONLY_ROLE)?;
    let main_readwrite_password = get_vault_password(&ytx_passwords, MAIN_READWRITE_ROLE)?;

    let admin_url = build_url(&postgres_url, &postgres_role, &postgres_password)?;
    let mut admin_client = postgres::Client::connect(&admin_url, postgres::NoTls)
        .context("Failed to connect to PostgreSQL server")?;

    create_database(&mut admin_client, &auth_db)?;
    create_database(&mut admin_client, &main_db)?;

    create_role(
        &mut admin_client,
        AUTH_READWRITE_ROLE,
        &auth_readwrite_password,
    )?;
    create_role(
        &mut admin_client,
        MAIN_READONLY_ROLE,
        &main_readonly_password,
    )?;
    create_role(
        &mut admin_client,
        MAIN_READWRITE_ROLE,
        &main_readwrite_password,
    )?;

    let auth_url = replace_postgres_url(&admin_url, &auth_db)?;
    let mut auth_client = postgres::Client::connect(&auth_url, postgres::NoTls)?;

    initialize_auth_database(&mut auth_client)?;
    insert_workspace_database(&mut auth_client, &main_workspace, &main_db)?;

    let main_url = replace_postgres_url(&admin_url, &main_db)?;
    let mut main_client = postgres::Client::connect(&main_url, postgres::NoTls)?;
    initialize_main_database(&mut main_client)?;

    grant_database_readwrite_permission(&mut auth_client, &auth_db, AUTH_READWRITE_ROLE)?;
    grant_database_readonly_permission(&mut main_client, &main_db, MAIN_READONLY_ROLE)?;
    grant_database_readwrite_permission(&mut main_client, &main_db, MAIN_READWRITE_ROLE)?;

    for section in SECTIONS {
        let readwrite_role = format!("ytx_main_{}_readwrite", section);
        let readonly_role = format!("ytx_main_{}_readonly", section);

        let readwrite_password = get_vault_password(&ytx_passwords, &readwrite_role)?;
        let readonly_password = get_vault_password(&ytx_passwords, &readonly_role)?;

        create_role(&mut admin_client, &readwrite_role, &readwrite_password)?;
        create_role(&mut admin_client, &readonly_role, &readonly_password)?;

        grant_section_readwrite_permission(&mut main_client, &main_db, section, &readwrite_role)?;
        grant_section_readonly_permission(&mut main_client, &main_db, section, &readonly_role)?;
    }

    Ok(())
}

fn read_vault_data(vault_addr: &str, token: &str, secret_path: &str) -> Result<Value> {
    let url = format!("{}/v1/{}", vault_addr.trim_end_matches('/'), secret_path);
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    let resp = Client::new().get(&url).headers(headers).send()?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP error {}", resp.status());
    }

    let json: Value = resp.json()?;
    Ok(json["data"]["data"].clone())
}

fn get_vault_password(data: &serde_json::Value, key: &str) -> Result<String> {
    data.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Vault key '{}' not found or not a string", key))
}

fn read_value_with_default(key: &str, default: &str) -> Result<String> {
    let val = var(key).unwrap_or(default.to_string());

    if val.is_empty() {
        bail!("Value for '{}' cannot be empty", key);
    }

    if val.len() > 63 {
        bail!("Value for '{}' cannot be longer than 63 characters", key);
    }

    let mut chars = val.chars();
    let first = chars.next().unwrap();

    if !first.is_ascii_lowercase() {
        bail!("Value for '{}' must start with a lowercase letter", key);
    }

    if !val
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        bail!(
            "Value for '{}' can only contain lowercase letters, digits, and underscore",
            key
        );
    }

    Ok(val)
}

fn read_workspace_with_default(key: &str, default: &str) -> Result<String> {
    let val = var(key).unwrap_or(default.to_string());

    if val.is_empty() {
        bail!("Value for '{}' cannot be empty", key);
    }

    if val.len() > 63 {
        bail!("Value for '{}' cannot be longer than 63 characters", key);
    }

    let mut chars = val.chars();
    let first = chars.next().unwrap();

    if !UnicodeXID::is_xid_start(first) {
        bail!(
            "Value for '{}' must start with a letter (Unicode allowed)",
            key
        );
    }

    if !val
        .chars()
        .all(|c| UnicodeXID::is_xid_continue(c) || c == '_')
    {
        bail!(
            "Value for '{}' can only contain letters, digits, or underscore",
            key
        );
    }

    Ok(val)
}
