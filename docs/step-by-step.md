# Weight-tracker step by step instructions

## Project init

Project started on github, cloned locally.

Open directory in cmd/powershell.

```cmd
cargo init
```

Open vscode in directory

```cmd
code .
```

## Setup of some chosen libraries

```cmd
cargo install diesel_cli --no-default-features --features "sqlite-bundled"
```

Create a new file named `.env` and store the following content:

```ini
DATABASE_URL=sqlite://./weight.sqlite
```

Run the setup. This will create the database file (weight.sqlite) and the `migrations` subdirectory

```cmd
diesel setup
```

Expand the `Cargo.toml` dependencies to include: `chrono`, `diesel`, `dotenv` and `rusqlite`.

```toml
[dependencies]
chrono = "0.4.23"
diesel = { version = "2.0.3", features = ["chrono", "sqlite"] }
dotenv = "0.15.0"
rusqlite = { version = "0.28.0", features = ["bundled"] }
```
