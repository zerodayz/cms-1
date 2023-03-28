# CMS 1

CMS 1 is a CMS written in Rust. It is a work in progress.

It is a hobby project to learn Rust and to create a CMS that is easy to use and easy to deploy.

The CMS 1 is aiming to be fully accessible.

## Deploy CMS 1

### Rust

You will need to have [Rust](https://www.rust-lang.org/) installed on your system.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup update
```

### Diesel

You will need to have [Diesel](https://diesel.rs/) installed on your system.

```bash
yum install mariadb-devel
cargo install diesel_cli --no-default-features --features mysql
```

### Podman

You will need to have [Podman](https://podman.io/) installed on your system.

### CMS-1

#### 1. Deploy MySQL database:

```bash
export MYSQL_ROOT_PASSWORD=roottoor
export MYSQL_DATABASE=cms
export MYSQL_USER=cms
export MYSQL_PASSWORD=cmspassword
podman run --name cms-backend-db -p 3307:3306/tcp -e MYSQL_ROOT_PASSWORD -e MYSQL_DATABASE -e MYSQL_USER -e MYSQL_PASSWORD -d mariadb:latest
```

#### 2. Setup the DB with diesel within `cms` directory:

```bash
export DATABASE_URL=mysql://cms:cmspassword@127.0.0.1:3307/cms
echo "DATABASE_URL=mysql://cms:cmspassword@127.0.0.1:3307/cms" > .env
diesel migration run
```

#### 3. Setup config.toml

```bash
[database]
# Instead of localhost, you have to use an IP address 127.0.0.1
host = "127.0.0.1"
port = 3307
user = "cms"
password = "cmspassword"
name = "cms"
```

#### 4. Create a user (usually first user):
```bash
cargo run --bin create_user
```

### Deploy CMS:

```bash
cargo run --release --package cms --bin cms
```

