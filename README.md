# Avaruuskapakka v2.0

![6n4re7v3](https://github.com/emepi/kapchan-v2/assets/149962304/2e041a0a-febf-4e70-b889-e799f3cedded)

## Build

### Backend

#### dependencies

- [rust](https://www.rust-lang.org/learn/get-started)
- [MySQL](https://dev.mysql.com/downloads/mysql/)
- [libmysqlclient-dev](https://dev.mysql.com/downloads/c-api/) (included in MySQL 8.0+)

#### Setup MySQL server

Open mysql command line client **(windows users should have this under programs menu after installation)**:

```bash
$ sudo mysql 
```

Create a new database:

```bash
mysql> CREATE DATABASE kapchan;
```

Create an user and grant database permissions:

```bash
mysql> CREATE USER 'username'@'localhost' IDENTIFIED BY 'password';
```

```bash
mysql> GRANT ALL PRIVILEGES ON kapchan.* TO 'username'@'localhost' WITH GRANT OPTION;
```

Copy *env.example* to *.env* and change database_url:

```bash
DATABASE_URL = mysql://username:password@127.0.0.1:3306/kapchan
```

#### Setup diesel cli tool & database schema

**On windows systems** add a new environment variable *MYSQLCLIENT_LIB_DIR* to equal 
*C:\Program Files\MySQL\MySQL Server 8.0\lib* or an equivalent installation path, and restart terminal before running this command.
<br>In the /backend directory:
```bash
$ cargo install diesel_cli --no-default-features --features mysql
```

```bash
$ diesel setup
```

#### Run the server

```bash
$ cargo run
```