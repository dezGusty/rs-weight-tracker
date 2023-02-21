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
