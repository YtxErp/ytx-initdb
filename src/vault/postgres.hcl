path "secret/data/postgres/*" {
  capabilities = ["create", "update", "read", "delete"]
}

path "secret/metadata/postgres/*" {
  capabilities = ["list"]
}
