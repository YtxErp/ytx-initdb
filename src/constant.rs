pub const FINANCE: &str = "finance";
pub const STAKEHOLDER: &str = "stakeholder";
pub const ITEM: &str = "item";
pub const TASK: &str = "task";
pub const SALE: &str = "sale";
pub const PURCHASE: &str = "purchase";

pub const SECTIONS: &[&str] = &[FINANCE, STAKEHOLDER, ITEM, TASK, SALE, PURCHASE];

pub const POSTGRES_SECRET_PATH: &str = "secret/data/postgres/postgres";
pub const YTX_SECRET_PATH: &str = "secret/data/postgres/ytx";

pub const AUTH_READWRITE_ROLE: &str = "ytx_auth_readwrite";
pub const MAIN_READWRITE_ROLE: &str = "ytx_main_readwrite";
pub const MAIN_READONLY_ROLE: &str = "ytx_main_readonly";
