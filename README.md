# Avaruuskapakka v2.0

![6n4re7v3](https://github.com/emepi/kapchan-v2/assets/149962304/2e041a0a-febf-4e70-b889-e799f3cedded)

## Installation
Step by step installation guide for running the latest kapchan-v2 imageboard software.

### Requirements
- MySQL 8.0+
- rust 1.88.0+
- libdav1d-dev [optional]

### Get the latest kapchan version
Start by cloning the main branch of this repository to get the latest version of kapchan-v2.

```
git clone https://github.com/emepi/kapchan-v2
```

### Setting up the database
Download the latest MySQL server and open the MySQL CLI: 

```
sudo mysql # mysql -u root -p (on windows systems)
```

Create a new database:

```
create database kapchan;
```

Create a MySQL user with custom username and password:

```
create user 'username'@'localhost' identified by 'password';
```

Grant the user permissions to access the database:

```
grant all on kapchan.* to 'username'@'localhost';
```

Exit the MySQL CLI by typing 'exit' and open up the kapchan-v2 project folder. <br>Inside the folder
copy the contents of '.env.example' to a new file '.env' and set a connection string to the 'DATABASE_URL' variable:

```
DATABASE_URL = mysql://username:password@127.0.0.1:3306/kapchan
```

Kapchan uses diesel ORM to manage the database schema.<br>
Make sure you have rust installed and install the diesel CLI tool:

```
cargo install diesel_cli --no-default-features --features mysql
```

Now you're ready to setup the database schema:

```
diesel migration run
```

### Build and run kapchan

Install the libdav1d-dev for av1 file support. If you wish to continue without av1 file support, you may remove the feature
"avif-native" of image crate dependency in cargo.toml.<br>
[Optional] Next, set unique ROOT_PASSWORD and COOKIE_SECRET values in .env file.
<br><br>
Run the kapchan package:
```
cargo run
```
Run with optimisations:
```
cargo build --release
```
```
cargo run --release
```