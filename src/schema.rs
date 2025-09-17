pub fn ytx_user() -> String {
    r#"
    CREATE TABLE IF NOT EXISTS ytx_user (
        id             UUID PRIMARY KEY,        -- Internal user identity (e.g., for business logic)
        email          TEXT UNIQUE,             -- User contact
        password_hash  TEXT NOT NULL,           -- Hashed user login password (bcrypt/argon2)
        register_time  TIMESTAMPTZ(0),
        updated_time   TIMESTAMPTZ(0),
        updated_by     UUID,
        last_login     TIMESTAMPTZ(0),
        is_valid       BOOLEAN DEFAULT TRUE
    );
    "#
    .to_string()
}

pub fn ytx_role_workspace() -> String {
    r#"
    CREATE TABLE IF NOT EXISTS ytx_role_workspace (
        user_id               UUID NOT NULL,                    -- Matches id in ytx_user (manually managed)
        role                  TEXT NOT NULL,
        workspace             TEXT NOT NULL,
        is_access_enabled     BOOLEAN DEFAULT FALSE,            -- Access to workspace
        register_time         TIMESTAMPTZ(0),
        updated_time          TIMESTAMPTZ(0),
        updated_by            UUID,
        is_valid              BOOLEAN DEFAULT TRUE,
        PRIMARY KEY (user_id, workspace)
    );
    "#
    .to_string()
}

pub fn ytx_workspace_database() -> String {
    r#"
    CREATE TABLE IF NOT EXISTS ytx_workspace_database (
        workspace        TEXT PRIMARY KEY,
        database         TEXT NOT NULL,
        created_time     TIMESTAMPTZ(0),
        updated_time     TIMESTAMPTZ(0),
        expire_time      TIMESTAMPTZ(0),
        updated_by       UUID,
        is_valid         BOOLEAN DEFAULT TRUE
    );
    "#
    .to_string()
}

pub fn ytx_meta() -> String {
    r#"
        CREATE TABLE IF NOT EXISTS ytx_meta (
            key TEXT PRIMARY KEY,
            value BOOLEAN,
            created_time TIMESTAMPTZ(0) DEFAULT now()
        );
        "#
    .to_string()
}

pub fn insert_meta() -> String {
    r#"
        INSERT INTO ytx_meta (key, value)
        VALUES ('ytx_managed', TRUE)
        ON CONFLICT (key) DO NOTHING;
        "#
    .to_string()
}

pub fn global_config() -> String {
    r#"
    CREATE TABLE IF NOT EXISTS global_config (
        section TEXT PRIMARY KEY,
        default_unit     INTEGER        DEFAULT 0,
        document_dir     TEXT           DEFAULT '',
        updated_time     TIMESTAMPTZ(0) DEFAULT now(),
        updated_by       UUID
    );
    "#
    .to_string()
}

pub fn insert_global_config(section: &str) -> String {
    format!(
        r#"
        INSERT INTO global_config (section)
        VALUES ('{}')
        ON CONFLICT (section) DO NOTHING;
        "#,
        section
    )
}

pub fn f_node_table() -> String {
    r#"
        CREATE TABLE IF NOT EXISTS finance_node (
            id               UUID PRIMARY KEY,
            name             TEXT,
            code             TEXT,
            description      TEXT,
            note             TEXT,
            kind             INTEGER,
            direction_rule   BOOLEAN DEFAULT FALSE,
            unit             INTEGER,
            initial_total    NUMERIC(16, 4),
            final_total      NUMERIC(16, 4),
            user_id          UUID,
            created_time     TIMESTAMPTZ(0),
            created_by       UUID,
            updated_time     TIMESTAMPTZ(0),
            updated_by       UUID,
            is_valid         BOOLEAN DEFAULT TRUE
        );
        "#
    .to_string()
}

pub fn f_entry_table() -> String {
    r#"
        CREATE TABLE IF NOT EXISTS finance_entry (
            id             UUID PRIMARY KEY,
            issued_time    TIMESTAMPTZ(0),
            code           TEXT,
            lhs_node       UUID,
            lhs_rate       NUMERIC(16, 8) CHECK (lhs_rate   >  0),
            lhs_debit      NUMERIC(12, 4) CHECK (lhs_debit  >= 0),
            lhs_credit     NUMERIC(12, 4) CHECK (lhs_credit >= 0),
            description    TEXT,
            document       TEXT,
            is_checked     BOOLEAN DEFAULT FALSE,
            rhs_credit     NUMERIC(16, 8) CHECK (rhs_credit >= 0),
            rhs_debit      NUMERIC(12, 4) CHECK (rhs_debit  >= 0),
            rhs_rate       NUMERIC(12, 4) CHECK (rhs_rate   >  0),
            rhs_node       UUID,
            user_id        UUID,
            created_time   TIMESTAMPTZ(0),
            created_by     UUID,
            updated_time   TIMESTAMPTZ(0),
            updated_by     UUID,
            is_valid       BOOLEAN DEFAULT TRUE
        );
        "#
    .to_string()
}

pub fn i_node_table() -> String {
    r#"
        CREATE TABLE IF NOT EXISTS item_node (
            id               UUID PRIMARY KEY,
            name             TEXT,
            code             TEXT,
            description      TEXT,
            note             TEXT,
            kind             INTEGER,
            direction_rule   BOOLEAN DEFAULT FALSE,
            unit             INTEGER,
            color            TEXT,
            unit_price       NUMERIC(16, 4),
            commission       NUMERIC(16, 4),
            initial_total    NUMERIC(16, 4),
            final_total      NUMERIC(16, 4),
            user_id          UUID,
            created_time     TIMESTAMPTZ(0),
            created_by       UUID,
            updated_time     TIMESTAMPTZ(0),
            updated_by       UUID,
            is_valid         BOOLEAN DEFAULT TRUE
        );
        "#
    .to_string()
}

pub fn i_entry_table() -> String {
    r#"
        CREATE TABLE IF NOT EXISTS item_entry (
            id             UUID PRIMARY KEY,
            issued_time    TIMESTAMPTZ(0),
            code           TEXT,
            lhs_node       UUID,
            unit_cost      NUMERIC(12, 4),
            lhs_debit      NUMERIC(12, 4) CHECK (lhs_debit >= 0),
            lhs_credit     NUMERIC(12, 4) CHECK (lhs_credit >= 0),
            description    TEXT,
            document       TEXT,
            is_checked     BOOLEAN DEFAULT FALSE,
            rhs_credit     NUMERIC(12, 4) CHECK (rhs_credit >= 0),
            rhs_debit      NUMERIC(12, 4) CHECK (rhs_debit >= 0),
            rhs_node       UUID,
            user_id        UUID,
            created_time   TIMESTAMPTZ(0),
            created_by     UUID,
            updated_time   TIMESTAMPTZ(0),
            updated_by     UUID,
            is_valid       BOOLEAN DEFAULT TRUE
        );
        "#
    .to_string()
}

pub fn t_node_table() -> String {
    r#"
        CREATE TABLE IF NOT EXISTS task_node (
            id               UUID PRIMARY KEY,
            name             TEXT,
            code             TEXT,
            description      TEXT,
            note             TEXT,
            kind             INTEGER,
            direction_rule   BOOLEAN DEFAULT FALSE,
            unit             INTEGER,
            issued_time      TIMESTAMPTZ(0),
            color            TEXT,
            document         TEXT,
            is_finished      BOOLEAN DEFAULT FALSE,
            initial_total    NUMERIC(16, 4),
            final_total      NUMERIC(16, 4),
            user_id          UUID,
            created_time     TIMESTAMPTZ(0),
            created_by       UUID,
            updated_time     TIMESTAMPTZ(0),
            updated_by       UUID,
            is_valid         BOOLEAN DEFAULT TRUE
        );
        "#
    .to_string()
}

pub fn t_entry_table() -> String {
    r#"
        CREATE TABLE IF NOT EXISTS task_entry (
            id             UUID PRIMARY KEY,
            issued_time    TIMESTAMPTZ(0),
            code           TEXT,
            lhs_node       UUID,
            unit_cost      NUMERIC(12, 4),
            lhs_debit      NUMERIC(12, 4) CHECK (lhs_debit >= 0),
            lhs_credit     NUMERIC(12, 4) CHECK (lhs_credit >= 0),
            description    TEXT,
            support_node   UUID,
            document       TEXT,
            is_checked     BOOLEAN DEFAULT FALSE,
            rhs_credit     NUMERIC(12, 4) CHECK (rhs_credit >= 0),
            rhs_debit      NUMERIC(12, 4) CHECK (rhs_debit >= 0),
            rhs_node       UUID,
            user_id        UUID,
            created_time   TIMESTAMPTZ(0),
            created_by     UUID,
            updated_time   TIMESTAMPTZ(0),
            updated_by     UUID,
            is_valid       BOOLEAN DEFAULT TRUE
        );
        "#
    .to_string()
}

pub fn s_node_table() -> String {
    r#"
        CREATE TABLE IF NOT EXISTS stakeholder_node (
            id               UUID PRIMARY KEY,
            name             TEXT,
            code             TEXT,
            description      TEXT,
            note             TEXT,
            kind             INTEGER,
            unit             INTEGER,
            payment_term     INTEGER,
            initial_total    NUMERIC(16, 4),
            final_total      NUMERIC(16, 4),
            user_id          UUID,
            created_time     TIMESTAMPTZ(0),
            created_by       UUID,
            updated_time     TIMESTAMPTZ(0),
            updated_by       UUID,
            is_valid         BOOLEAN DEFAULT TRUE
        );
        "#
    .to_string()
}

pub fn s_entry_table() -> String {
    r#"
        CREATE TABLE IF NOT EXISTS stakeholder_entry (
            id                 UUID PRIMARY KEY,
            issued_time        TIMESTAMPTZ(0),
            code               TEXT,
            lhs_node           UUID,
            unit_price         NUMERIC(12, 4),
            description        TEXT,
            external_item      UUID,
            document           TEXT,
            is_checked         BOOLEAN DEFAULT FALSE,
            rhs_node           UUID,
            user_id            UUID,
            created_time       TIMESTAMPTZ(0),
            created_by         UUID,
            updated_time       TIMESTAMPTZ(0),
            updated_by         UUID,
            is_valid           BOOLEAN DEFAULT TRUE,
            UNIQUE(lhs_node, rhs_node)
        );
        "#
    .to_string()
}

pub fn o_node_table(order: &str) -> String {
    format!(
        r#"
        CREATE TABLE IF NOT EXISTS {}_node (
            id                UUID PRIMARY KEY,
            name              TEXT,
            description       TEXT,
            kind              INTEGER,
            direction_rule    BOOLEAN DEFAULT FALSE,
            unit              INTEGER,
            party             UUID,
            employee          UUID,
            issued_time       TIMESTAMPTZ(0),
            first_total       NUMERIC(16, 4),
            second_total      NUMERIC(16, 4),
            is_finished       BOOLEAN DEFAULT FALSE,
            initial_total     NUMERIC(16, 4),
            discount_total    NUMERIC(16, 4),
            final_total       NUMERIC(16, 4),
            settlement_node   UUID,
            user_id           UUID,
            created_time      TIMESTAMPTZ(0),
            created_by        UUID,
            updated_time      TIMESTAMPTZ(0),
            updated_by        UUID,
            is_valid          BOOLEAN DEFAULT TRUE
        );
        "#,
        order
    )
}

pub fn o_entry_table(order: &str) -> String {
    format!(
        r#"
        CREATE TABLE IF NOT EXISTS {}_entry (
            id                  UUID PRIMARY KEY,
            lhs_node            UUID,
            unit_price          NUMERIC(12, 4),
            first               NUMERIC(12, 4),
            second              NUMERIC(12, 4),
            description         TEXT,
            external_item       UUID,
            discount            NUMERIC(12, 4),
            final               NUMERIC(12, 4),
            initial             NUMERIC(12, 4),
            discount_price      NUMERIC(12, 4),
            rhs_node            UUID,
            user_id             UUID,
            created_time        TIMESTAMPTZ(0),
            created_by          UUID,
            updated_time        TIMESTAMPTZ(0),
            updated_by          UUID,
            is_valid            BOOLEAN DEFAULT TRUE
        );
        "#,
        order
    )
}

pub fn o_settlement_table(order: &str) -> String {
    format!(
        r#"
            CREATE TABLE IF NOT EXISTS {}_settlement (
                id               UUID PRIMARY KEY,
                party            UUID,
                issued_time      TIMESTAMPTZ(0),
                description      TEXT,
                is_finished      BOOLEAN DEFAULT FALSE,
                initial_total    NUMERIC(16, 4),
                user_id          UUID,
                created_time     TIMESTAMPTZ(0),
                created_by       UUID,
                updated_time     TIMESTAMPTZ(0),
                updated_by       UUID,
                is_valid         BOOLEAN DEFAULT TRUE
            );
            "#,
        order
    )
}

pub fn path_table(table_name: &str) -> String {
    format!(
        r#"
        CREATE TABLE IF NOT EXISTS {}_path (
            ancestor      UUID,
            descendant    UUID,
            distance      INTEGER DEFAULT 1 CHECK (distance = 1),
            PRIMARY KEY (ancestor, descendant)
        );
        "#,
        table_name
    )
}
