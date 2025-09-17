# YTX InitDB

## Overview

**YTX InitDB** is the database initialization tool for the **YTX ERP system**.

It automates:

- Creation of PostgreSQL databases and roles (both global and section-specific).
- Setup of granular permissions.
- Schema initialization.

All passwords are securely fetched from **Vault**, ensuring no secrets are stored in code or environment files.

## Features

- **Database creation**: Databases are created based on `.env` configuration.
  - `AUTH_DB` → Authentication database (default: `ytx_auth`)
  - `MAIN_DB` → Main application database (default: `ytx_main`)
  - `MAIN_WORKSPACE` → Default workspace identifier for the main DB (default: `ytx_workspace`)

- **Role management**: All YTX roles are **fixed** and created automatically.

| Section       | Readonly Role                  | ReadWrite Role                 | Description                                      |
|---------------|--------------------------------|--------------------------------|--------------------------------------------------|
| Main DB       | ytx_main_readonly              | ytx_main_readwrite             | Role for main database, global                   |
| Auth DB       | -                              | ytx_auth_readwrite             | Role for authentication database                 |
| Finance       | ytx_main_finance_readonly      | ytx_main_finance_readwrite     | Finance section in MAIN_DB                       |
| Stakeholder   | ytx_main_stakeholder_readonly  | ytx_main_stakeholder_readwrite | Stakeholder section in MAIN_DB                   |
| Item          | ytx_main_item_readonly         | ytx_main_item_readwrite        | Item section in MAIN_DB                          |
| Task          | ytx_main_task_readonly         | ytx_main_task_readwrite        | Task section in MAIN_DB                          |
| Sale          | ytx_main_sale_readonly         | ytx_main_sale_readwrite        | Sale section in MAIN_DB                          |
| Purchase      | ytx_main_purchase_readonly     | ytx_main_purchase_readwrite    | Purchase section in MAIN_DB                      |

- Schema and essential data initialization
- Granular role permissions for secure data access

## Technology Stack

- **Language:** Rust
- **Database:** PostgreSQL (`postgres` crate)
- **Secret Management:** Vault (`reqwest` crate for HTTP API)
- **Config:** `.env` file loaded via `dotenvy`
- **Vault:** KV v2 secrets engine, JSON-formatted secret data

## Environment Variables (`.env`)

| Variable           | Default           | Description                                                                                  |
|--------------------|-------------------|----------------------------------------------------------------------------------------------|
| `POSTGRES_ROLE`    | `postgres`        | Global PostgreSQL superuser role; **can be customized via `.env`**. Used to create all YTX databases and roles. Ensure it has sufficient privileges. |
| `AUTH_DB`          | `ytx_auth`        | Authentication database. Can be customized.                                                  |
| `MAIN_DB`          | `ytx_main`        | Main application database. Can be customized.                                                |
| `MAIN_WORKSPACE`   | `ytx_workspace`   | Default workspace identifier for the main DB. **⚠️ Must be unique**: each workspace should correspond to exactly one MAIN_DB.  |
| `VAULT_TOKEN`      | *(none)*                                      | Vault token used to fetch passwords. Must be provided in `.env`.                                   |
| `VAULT_URL`        | `http://127.0.0.1:8200`                       | Vault server address. Can be customized.                                                           |
| `POSTGRES_URL`     | `postgres://postgres@localhost:5432/postgres` | Connection URL for PostgreSQL superuser. Can be customized.                                        |

## Security & Password Management

- Vault token (`VAULT_TOKEN`) is required to fetch passwords.
- **No passwords** are hardcoded or stored in .env.
- Required Vault secrets must exist **before** running InitDB:

| Secret Path                | Keys                                                                                                                                                         |
|----------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `secret/postgres/postgres` | `postgres` (PostgreSQL superuser password)                                                                                                                   |
| `secret/postgres/ytx`      | `ytx_auth_readwrite`, `ytx_main_readwrite`, `ytx_main_readonly`, `ytx_main_finance_readwrite`, `ytx_main_finance_readonly`, `ytx_main_stakeholder_readwrite`, `ytx_main_stakeholder_readonly`, `ytx_main_item_readwrite`, `ytx_main_item_readonly`, `ytx_main_task_readwrite`, `ytx_main_task_readonly`, `ytx_main_sale_readwrite`, `ytx_main_sale_readonly`, `ytx_main_purchase_readwrite`, `ytx_main_purchase_readonly` |

- If any secret is missing, InitDB will fail.

## Quick Start

### 1. Run PostgreSQL & Vault with Docker

A preconfigured `docker-compose.yml` is provided for local development/testing.

- **PostgreSQL**: persistent storage, configurable password
- **Vault**: local file storage, UI enabled, port mapping
- **Important**: Always wrap `POSTGRES_PASSWORD` in double quotes (`""`) in Docker Compose.
- **Volumes** are used to persist database and Vault data outside the container. Customize these paths on your host machine as needed.

| Container          | Volume Path (Host)                | Container Path                | Purpose                                                                                              |
|:------------------:|-----------------------------------|-------------------------------|------------------------------------------------------------------------------------------------------|
| PostgreSQL         | `/path/to/your/local/postgres`    | `/var/lib/postgresql/data`    | Stores all database files; ensures DB data persists across container restarts. |
| Vault file         | `/path/to/your/local/vault/file`  | `/vault/file`                 | Stores Vault's persistent secrets and configuration; ensures Vault data survives container restarts. |
| Vault logs         | `/path/to/your/local/vault/logs`  | `/vault/logs`                 | Stores Vault log files; useful for debugging and auditing. |

```bash
docker compose -p ytx up -d
```

---

### 2. Configure Environment & Vault

- Copy the environment template:

  ```shell
  cp env_template.text .env
  ```

- Store PostgreSQL superuser password in Vault:

  ```shell
  vault kv put secret/postgres/postgres postgres=POSTGRES_PASSWORD
  ```

- Generate and store random passwords for all YTX roles in Vault:

  ```shell
  vault kv patch/put secret/postgres/ytx \
    ytx_auth_readwrite=$(openssl rand -base64 16) \
    ytx_main_readwrite=$(openssl rand -base64 16) \
    ytx_main_readonly=$(openssl rand -base64 16) \
    ytx_main_finance_readwrite=$(openssl rand -base64 16) \
    ytx_main_finance_readonly=$(openssl rand -base64 16) \
    ytx_main_stakeholder_readwrite=$(openssl rand -base64 16) \
    ytx_main_stakeholder_readonly=$(openssl rand -base64 16) \
    ytx_main_item_readwrite=$(openssl rand -base64 16) \
    ytx_main_item_readonly=$(openssl rand -base64 16) \
    ytx_main_task_readwrite=$(openssl rand -base64 16) \
    ytx_main_task_readonly=$(openssl rand -base64 16) \
    ytx_main_sale_readwrite=$(openssl rand -base64 16) \
    ytx_main_sale_readonly=$(openssl rand -base64 16) \
    ytx_main_purchase_readwrite=$(openssl rand -base64 16) \
    ytx_main_purchase_readonly=$(openssl rand -base64 16)

---

### 3. Initialize Database

```shell
git clone https://github.com/YtxErp/ytx-initdb.git
cd ytx-initdb

cargo run --release
```

---

### 4. Verify

```shell
psql -h localhost -U <postgres_user> -d <database_name>
# Example:
psql -h localhost -U postgres -d ytx_auth
psql -h localhost -U postgres -d ytx_main
```

---

## Configuration Reference

- Vault address and tokens are provided via environment variables.
- Database and workspace names are customizable.
- Each workspace should have a unique main database for data isolation.

---

## Support

If YTX has been helpful to you, I’d be truly grateful for your support. Your encouragement helps me keep improving and creating!

Also may the force be with you.

[<img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" width="160" height="40">](https://buymeacoffee.com/ytx.cash)
