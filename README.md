# YTX InitDB

## Overview

**YTX InitDB** is a database initialization tool for the YTX ERP system.

It automates the creation of PostgreSQL databases and all required roles (global and section-specific), configures granular permissions, and initializes schemas.

All passwords are securely fetched from Vault, ensuring no secrets are stored in code or environment files

---

## Features

- Automated creation of PostgreSQL databases (e.g., `ytx_auth`, `ytx_main`)
- Automated creation of roles, including:
  - Global roles:
    - `ytx_auth_readwrite`
    - `ytx_main_readwrite`
    - `ytx_main_readonly`
  - Section-specific roles (Main DB):
    - Finance: `ytx_main_finance_readwrite`, `ytx_main_finance_readonly`
    - Stakeholder: `ytx_main_stakeholder_readwrite`, `ytx_main_stakeholder_readonly`
    - Item: `ytx_main_item_readwrite`, `ytx_main_item_readonly`
    - Task: `ytx_main_task_readwrite`, `ytx_main_task_readonly`
    - Sale: `ytx_main_sale_readwrite`, `ytx_main_sale_readonly`
    - Purchase: `ytx_main_purchase_readwrite`, `ytx_main_purchase_readonly`
- Schema and essential data initialization
- Granular role permissions for secure data access

---

## Technology Stack

- **Language:** Rust
- **Database:** PostgreSQL (`postgres` crate)
- **Secret Management:** Vault (`reqwest` crate for HTTP API)
- **Config:** `.env` file loaded via `dotenvy`
- **Vault:** KV v2 secrets engine, JSON-formatted secret data

---

## Password Management & Security

- Vault token (`VAULT_TOKEN`) is required to fetch passwords.
- All roles MUST have their corresponding passwords pre-created and stored in Vault before InitDb can run.

- Vault secret paths:
  - PostgreSQL superuser password
    - `secret/data/postgres/postgres`
  - YTX roles (auth, main, section-specific):
    - `secret/data/postgres/ytx`

- Required roles:
  - PostgreSQL superuser: `postgres`
  - Auth DB: `ytx_auth_readwrite`
  - Main DB: `ytx_main_readwrite`, `ytx_main_readonly`
  - Section-specific roles (Main DB):
    - finance: `ytx_main_finance_readwrite`, `ytx_main_finance_readonly`
    - stakeholder: `ytx_main_stakeholder_readwrite`, `ytx_main_stakeholder_readonly`
    - item: `ytx_main_item_readwrite`, `ytx_main_item_readonly`
    - task: `ytx_main_task_readwrite`, `ytx_main_task_readonly`
    - sale: `ytx_main_sale_readwrite`, `ytx_main_sale_readonly`
    - purchase: `ytx_main_purchase_readwrite`, `ytx_main_purchase_readonly`

- Without these Vault secrets, InitDb will fail.

---

## Quick Start

### 1. Run PostgreSQL & Vault with Docker

A preconfigured `docker-compose.yml` is provided for local development/testing.

- **PostgreSQL**: persistent storage, configurable password
- **Vault**: local file storage, UI enabled, port mapping
- **Important**: Always wrap `POSTGRES_PASSWORD` in double quotes (`""`) in Docker Compose.
- **Volumes** are used to persist database and Vault data outside the container. Customize these paths on your host machine as needed.

| Container          | Volume Path (Host)                | Container Path                | Purpose |
|:------------------:|-----------------------------------|-------------------------------|---------|
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
