use crate::constant::*;
use crate::schema::*;

use postgres::Client;
use url::Url;

pub fn create_database(
    client: &mut Client,
    db_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let exists: bool = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)",
            &[&db_name],
        )?
        .get(0);

    if !exists {
        let create_sql = format!("CREATE DATABASE {}", db_name);
        client.execute(&create_sql, &[])?;
        println!("Database {} created.", db_name);
    } else {
        println!("Database {} already exists.", db_name);
    }

    Ok(())
}

pub fn create_user(
    client: &mut Client,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let exists: bool = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM pg_roles WHERE rolname = $1)",
            &[&username],
        )?
        .get(0);

    if !exists {
        let row = client.query_one("SELECT quote_literal($1)", &[&password])?;
        let escaped_password: String = row.get(0);

        let sql = format!(
            "CREATE ROLE {} WITH LOGIN PASSWORD {} NOCREATEDB NOCREATEROLE",
            username, escaped_password
        );

        client.execute(&sql, &[])?;
        println!("User {} created.", username);
    } else {
        println!("User {} already exists.", username);
    }

    Ok(())
}

pub fn initialize_main_database(client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    let mut transaction = client.transaction()?;

    let mut sqls = Vec::new();

    sqls.extend([
        ytx_meta(),
        global_config(),
        f_node_table(),
        s_node_table(),
        i_node_table(),
        t_node_table(),
        f_entry_table(),
        s_entry_table(),
        t_entry_table(),
        i_entry_table(),
    ]);

    for section in SECTIONS {
        sqls.push(path_table(section));
        sqls.push(insert_global_config(section));
    }

    for section in [SALE, PURCHASE] {
        sqls.push(o_node_table(section));
        sqls.push(o_entry_table(section));
        sqls.push(o_settlement_table(section));
    }

    sqls.push(insert_meta());

    for sql in sqls {
        if let Err(e) = transaction.execute(&sql, &[]) {
            let _ = transaction.rollback();
            return Err(format!("Failed to execute SQL `{sql}`: {e}").into());
        }
    }

    transaction.commit()?;
    Ok(())
}

pub fn initialize_auth_database(client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    let mut transaction = client.transaction()?;

    let mut sqls = Vec::new();

    sqls.extend([
        ytx_user(),
        ytx_user_workspace(),
        ytx_workspace_database(),
        ytx_database_role(),
    ]);

    for sql in sqls {
        if let Err(e) = transaction.execute(&sql, &[]) {
            let _ = transaction.rollback();
            return Err(format!("Failed to execute SQL `{sql}`: {e}").into());
        }
    }

    transaction.commit()?;
    Ok(())
}

pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if name.len() > 63 {
        return Err("Name cannot be longer than 63 characters".to_string());
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    if !first.is_ascii_lowercase() {
        return Err("Name must start with a lowercase letter".to_string());
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err("Name can only contain lowercase letters, digits, and underscore".to_string());
    }

    Ok(())
}

pub fn grant_readonly_permission(
    postgres_client: &mut Client,
    main_client: &mut Client,
    db_name: &str,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let grant_connect = format!("GRANT CONNECT ON DATABASE {} TO {}", db_name, username);
    postgres_client.execute(&grant_connect, &[])?;

    let grant_usage = format!("GRANT USAGE ON SCHEMA public TO {}", username);
    main_client.execute(&grant_usage, &[])?;

    let grant_select = format!(
        "GRANT SELECT ON ALL TABLES IN SCHEMA public TO {}",
        username
    );
    main_client.execute(&grant_select, &[])?;

    let default_privileges = format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO {}",
        username
    );
    main_client.execute(&default_privileges, &[])?;

    Ok(())
}

pub fn grant_readwrite_permission(
    postgres_client: &mut Client,
    main_client: &mut Client,
    db_name: &str,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let grant_connect = format!("GRANT CONNECT ON DATABASE {} TO {}", db_name, username);
    postgres_client.execute(&grant_connect, &[])?;

    let grant_usage = format!("GRANT USAGE ON SCHEMA public TO {}", username);
    main_client.execute(&grant_usage, &[])?;

    let grant_all = format!(
        "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO {}",
        username
    );
    main_client.execute(&grant_all, &[])?;

    let grant_sequences = format!(
        "GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO {}",
        username
    );
    main_client.execute(&grant_sequences, &[])?;

    let default_privileges = format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO {}",
        username
    );
    main_client.execute(&default_privileges, &[])?;

    let default_priv_sequences = format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO {}",
        username
    );
    main_client.execute(&default_priv_sequences, &[])?;

    Ok(())
}

pub fn grant_admin_permission(
    postgres_client: &mut Client,
    client: &mut Client,
    db_name: &str,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let grant_global = format!("ALTER ROLE {} WITH CREATEDB CREATEROLE LOGIN", username);
    postgres_client.execute(&grant_global, &[])?;

    let grant_connect = format!("GRANT CONNECT ON DATABASE {} TO {}", db_name, username);
    postgres_client.execute(&grant_connect, &[])?;

    client.execute(
        &format!("GRANT USAGE ON SCHEMA public TO {}", username),
        &[],
    )?;
    client.execute(
        &format!(
            "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO {}",
            username
        ),
        &[],
    )?;
    client.execute(
        &format!(
            "GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO {}",
            username
        ),
        &[],
    )?;

    client.execute(
        &format!(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON TABLES TO {}",
            username
        ),
        &[],
    )?;
    client.execute(&format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO {}",
        username
    ), &[])?;

    Ok(())
}

pub fn replace_postgres_url(
    postgres_url: &str,
    new_db: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut url = Url::parse(postgres_url)?;
    url.set_path(&format!("/{}", new_db));
    Ok(url.to_string())
}

pub fn insert_workspace_database(
    client: &mut Client,
    workspace_name: &str,
    database_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let row = client.query_opt(
        "SELECT database_name FROM ytx_workspace_database WHERE workspace_name = $1",
        &[&workspace_name],
    )?;

    if let Some(row) = row {
        let existing_db: String = row.get(0);
        if existing_db == database_name {
            return Ok(());
        } else {
            return Err(format!(
            "Workspace '{}' is already linked to a different database '{}', please check your .env configuration.",
            workspace_name, existing_db
        ).into());
        }
    }

    let sql = r#"
    INSERT INTO ytx_workspace_database (workspace_name, database_name)
    VALUES ($1, $2);
"#;

    client.execute(sql, &[&workspace_name, &database_name])?;
    println!(
        "Workspace '{}' linked to database '{}'",
        workspace_name, database_name
    );

    Ok(())
}

pub fn insert_database_role(
    client: &mut Client,
    database_name: &str,
    role_name: &str,
    password_enc: &str,
    nonce: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let sql = r#"
        INSERT INTO ytx_database_role (database_name, role_name, password_enc, nonce)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (database_name, role_name) DO UPDATE
        SET password_enc = EXCLUDED.password_enc,
            nonce = EXCLUDED.nonce,
            updated_time = now()
    "#;

    client.execute(sql, &[&database_name, &role_name, &password_enc, &nonce])?;
    Ok(())
}

pub fn grant_auth_read_permission(
    postgres_client: &mut Client,
    auth_client: &mut Client,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Grant CONNECT privilege on the ytx_auth database
    let grant_connect = format!("GRANT CONNECT ON DATABASE ytx_auth TO {}", username);
    postgres_client.execute(&grant_connect, &[])?;

    // Grant USAGE privilege on the public schema
    let grant_usage = format!("GRANT USAGE ON SCHEMA public TO {}", username);
    auth_client.execute(&grant_usage, &[])?;

    // Grant SELECT privilege on all tables in the public schema
    let grant_select = format!(
        "GRANT SELECT ON ALL TABLES IN SCHEMA public TO {}",
        username
    );
    auth_client.execute(&grant_select, &[])?;

    // Set default privileges to automatically grant SELECT on new tables in public schema
    let alter_default = format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO {}",
        username
    );
    auth_client.execute(&alter_default, &[])?;

    Ok(())
}
