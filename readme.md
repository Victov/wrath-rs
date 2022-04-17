# Wrath-rs: World of Warcraft: Wotlk server emulator in pure rust.
This is an educational project to create  a server emulator for World of Warcraft patch 3.3.5 (12340) in Rust. It uses `async-std` to remain fully asynchronous for maximum performance. It is nowhere near playable, let alone feature complete right now. Contributions in the form of pull requests are welcome, though there is currently no well-organised backlog of issues or workflow. 

## Current Feature Status
- [x] Login and character creation.
- [x] Getting into the world. 
- [x] Movement
- [ ] Equipment
- [ ] Creatures

## Getting Started
Install Rust and clone the repo. Set up a MySQL server. wrath-rs requires a MySQL connection to manage its data. Queries in code are checked at compile-time with a live database connection, so a fully set-up database is required in order to compile the project. 
### Database Setup
Get a MySQL server up and running. Install `cargo sqlx` by following [these instructions](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli)

Navigate into the `databases/wrath-auth-db` folder, copy the `.env.template` file into `.env` and open the `.env` file with your favourite text editor. Change the connection data to match your local MySQL server. Create the database and run migrations through `sqlx-cli` commands. 
```
cd databases/wrath-auth-db
cp .env.template .env
#modify .env file to match your database setup
cargo sqlx database create
cargo sqlx migrate run
```
With your favourite database browser you can now verify that the authentication database has been created and some testing accounts have been inserted to get you started. Repeat this process for the `databases/wrath-realm-db` folder. Verify that also the world database has been set up.
### Compiling the project
Navigate to the `auth_server` folder, set up the `.env` file from provided `.env.template` and you are ready to compile and run the Authentication server.
```
cd auth_server
cp .env.template .env
#modify .env with your favourite text editor
cargo run
```
Repeat these steps for the `world_server` folder to kick off a world server. You should now be able to log in with user `test` with password `test` using a 3.3.5(12340) game client and create your first character.

### After initial setup
Some progress on the server code may change the database tables. In that case you will have to go into the database folders and run to bring your database up to the latest structure. This will wipe your database. This shouldn't be an issue since the server is nowhere near being able to host actual players anyway. 
```
cargo sqlx database drop
cargo sqlx database create
cargo sqlx migrate run
``` 

Hint: Windows users can run `reset_db.bat` to quickly reset both the authentication and world databases. This requires `sqlx-cli` to be correctly installed.

## Console Commands
Both the authentication server and the world server accept console commands to be typed while they're running. This is useful to control certain aspects of the servers and databases, without having to resort to third-party database editing tools. Currently available console commands are:

### On the auth server
| Command                                | Description                                                                           |
|----------------------------------------|---------------------------------------------------------------------------------------|
| `create-account <username> <password>` | Inserts a fresh user into the database with the given username and password.          |
| `ban <username>`                       | Bans a user in the database (does not currently disconnect them if they're connected) |
| `unban <username>`                     | Unbans a user.                                                                        |

### On the world server
| Command                                | Description                                                                           |
|----------------------------------------|---------------------------------------------------------------------------------------|
| `exit` 				 | Gracefully shuts down the world server. 						 | 
