cd databases/wrath-auth-db/
cargo sqlx database reset -y
cargo sqlx prepare
cd ../../databases/wrath-realm-db
cargo sqlx database reset -y
cargo sqlx prepare


