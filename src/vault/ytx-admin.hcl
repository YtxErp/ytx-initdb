path "secret/data/postgres/ytx" {
  capabilities = ["create", "update", "read", "delete"]
}

path "secret/metadata/postgres/ytx" {
  capabilities = ["list"]
}