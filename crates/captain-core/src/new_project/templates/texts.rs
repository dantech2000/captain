//! The `compose.yaml` text of each template. `$$` keeps a `$` for the shell in
//! the container; `${NAME:?...}` reads `.env`.

pub const POSTGRES: &str = r#"# PostgreSQL from Captain's template. The password is in .env.
name: {name}
services:
  postgres:
    image: postgres:18
    restart: unless-stopped
    environment:
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD in .env}
    ports:
      - "{port0}:5432"
    volumes:
      # From 18 on, the image keeps its data in /var/lib/postgresql/18/docker.
      - data:/var/lib/postgresql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5
volumes:
  data: {}
x-captain:
  tasks:
    databases:
      service: postgres
      command: psql -U postgres -c '\l'
    db-size:
      service: postgres
      command: psql -U postgres -c "select datname, pg_size_pretty(pg_database_size(datname)) as size from pg_database"
"#;

pub const MYSQL: &str = r#"# MySQL from Captain's template. The root password is in .env.
name: {name}
services:
  mysql:
    image: mysql:8.4
    restart: unless-stopped
    environment:
      MYSQL_ROOT_PASSWORD: ${MYSQL_ROOT_PASSWORD:?set MYSQL_ROOT_PASSWORD in .env}
    ports:
      - "{port0}:3306"
    volumes:
      - data:/var/lib/mysql
volumes:
  data: {}
x-captain:
  tasks:
    databases:
      service: mysql
      command: mysql -uroot -p"$$MYSQL_ROOT_PASSWORD" -e "show databases"
"#;

pub const REDIS: &str = r#"# Redis from Captain's template. It has no password, so keep the port local.
name: {name}
services:
  redis:
    image: redis:8-alpine
    restart: unless-stopped
    command: ["redis-server", "--appendonly", "yes"]
    ports:
      - "{port0}:6379"
    volumes:
      - data:/data
volumes:
  data: {}
x-captain:
  tasks:
    keys:
      service: redis
      command: redis-cli dbsize
    memory:
      service: redis
      command: redis-cli info memory
"#;

pub const MONGO: &str = r#"# MongoDB from Captain's template. The root user and password are in .env.
name: {name}
services:
  mongo:
    image: mongo:8
    restart: unless-stopped
    environment:
      MONGO_INITDB_ROOT_USERNAME: ${MONGO_INITDB_ROOT_USERNAME:?set MONGO_INITDB_ROOT_USERNAME in .env}
      MONGO_INITDB_ROOT_PASSWORD: ${MONGO_INITDB_ROOT_PASSWORD:?set MONGO_INITDB_ROOT_PASSWORD in .env}
    ports:
      - "{port0}:27017"
    volumes:
      - data:/data/db
volumes:
  data: {}
x-captain:
  tasks:
    databases:
      service: mongo
      command: mongosh --quiet -u "$$MONGO_INITDB_ROOT_USERNAME" -p "$$MONGO_INITDB_ROOT_PASSWORD" --eval "db.adminCommand('listDatabases').databases.forEach(d => print(d.name))"
"#;

pub const RABBITMQ: &str = r#"# RabbitMQ from Captain's template. The user and password are in .env.
name: {name}
services:
  rabbitmq:
    image: rabbitmq:4-management
    restart: unless-stopped
    # The node name and the data folder come from the host name, so it stays fixed.
    hostname: rabbitmq
    environment:
      RABBITMQ_DEFAULT_USER: ${RABBITMQ_DEFAULT_USER:?set RABBITMQ_DEFAULT_USER in .env}
      RABBITMQ_DEFAULT_PASS: ${RABBITMQ_DEFAULT_PASS:?set RABBITMQ_DEFAULT_PASS in .env}
    ports:
      - "{port0}:5672"
      # The management web page.
      - "{port1}:15672"
    volumes:
      - data:/var/lib/rabbitmq
volumes:
  data: {}
x-captain:
  tasks:
    queues:
      service: rabbitmq
      command: rabbitmqctl list_queues name messages consumers
"#;

pub const NGINX: &str = r#"# A web server from Captain's template. Put your files in the site folder.
name: {name}
services:
  web:
    image: nginx:stable-alpine
    restart: unless-stopped
    ports:
      - "{port0}:80"
    volumes:
      - ./site:/usr/share/nginx/html:ro
"#;

pub const INDEX_HTML: &str = "<!doctype html>
<html>
  <head><meta charset=\"utf-8\"><title>It works</title></head>
  <body>
    <h1>It works</h1>
    <p>Edit site/index.html in this project folder.</p>
  </body>
</html>
";
