cd databases/wrath-auth-db/
cargo sqlx database reset -y
cd ../../databases/wrath-realm-db
cargo sqlx database reset -y

