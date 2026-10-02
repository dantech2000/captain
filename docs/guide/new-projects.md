# Making new projects

The **New** sheet makes a Compose project in four ways. Open it with the **+** beside Projects in the sidebar, with ⌘N (Ctrl N on Linux and Windows), or with `new` in the ⌘K palette.

Each way ends on the project's **Files** tab with `compose.yaml` open. Nothing starts there. Click **Save and apply** to see what `docker compose up` will create, then click **Apply** to start it. See [Editing a project's files](editing-projects.md).

In the sheet, Up and Down move between the four cards, Return opens one, and ⌘1 to ⌘4 open a card directly. In a form, Tab and Shift Tab move between the fields, ⌘Return runs the main button, and Escape goes back one step, like Back. On the four cards, Escape closes the sheet.

## Where projects go

Captain writes a new project to `~/Captain/<name>/`. The `projects_dir` setting in the settings file changes that folder; see [Settings](settings.md). A relative path in `projects_dir`, such as `projects`, is inside your home folder. Captain creates the folder on first use. **Show folder** at the bottom of the sheet opens it.

The project name is also the folder name and the Compose project name. It has only lowercase letters, digits, `-`, and `_`, and starts with a letter or a digit. Captain suggests a free name, such as `postgres-2` when `postgres` exists. Captain never writes into a folder that has files, or into a link. Pick another name instead. Captain writes all the files first, then moves the finished folder into place, so a failure leaves no half-made project.

## Projects Captain remembers

Captain keeps the projects it made or opened in `~/.captain/projects.json`. Each entry has the project name, its folder, and its Compose files. Because of this list, a project stays in the sidebar after **Down**, as **Stopped**, with an **Up** button. `up <name>` in the ⌘K palette starts it too.

Captain replaces the file in one step, so a crash leaves the old list or the new one. Captain reads the file again before each change, so your own edits stay. If the file is not valid JSON, Captain does not change it until you fix or delete it, and the New sheet refuses to make a project before it writes any files.

To forget a project, right-click it in the sidebar and choose **Remove from Captain**. Captain removes the entry from `projects.json` only. The folder, its files, the containers, and the volumes stay. A project that still runs stays in the sidebar, because Captain finds it from its containers.

## Run an image

1. Choose **Run an image**.
2. Type in the search field. The list shows the images on this engine first, then Docker Hub. Official images have an **Official** tag, and each Docker Hub row shows its stars and pulls. The first row offers the text you typed, for an image the search does not find, such as one in a private registry.
3. Pick an image with Return or a click.
4. Set the tag. The chips under the field are Docker Hub's newest tags, or the engine's tags for a local image.
5. If the engine has the image, Captain fills in a port row for each port the image exposes, with a free port on this computer. If it does not, click **Pull** to download it and fill in the ports.
6. Add or remove port, environment, and volume rows. A volume source is a volume name, such as `data`, or a folder. A plain word such as `site` is a volume name. To use the project's `site` folder, write `./site`. An absolute folder, such as `/srv/site` or `C:\work\site` on Windows, works too.
7. Pick a restart policy.
8. Click **Create project**.

**Save as a project** is on by default. Captain then writes a `compose.yaml` with one service. Click **compose.yaml** above the form to see the file before Captain writes it.

An environment variable whose name has `PASSWORD`, `SECRET`, `TOKEN`, `PASSWD`, `API_KEY`, or `PRIVATE` in it goes in `.env`, not in `compose.yaml`, as for templates (see [Passwords and .env](#passwords-and-env)). `compose.yaml` reads it with `${NAME:?...}`, so the preview shows no secret. A value with `'`, `\`, or a line break cannot go in `.env` this way; leave it empty and set it in `.env` yourself.

If you pick an image with a digest, such as `app@sha256:...`, the form keeps the digest next to the tag, and the project runs that exact image. Change the tag to drop the digest.

Turn **Save as a project** off to run a plain container now, like `docker run -d`. The name field is then the container name. Captain pulls the image first if the engine does not have it. A plain container has no file, so a folder volume does not work there; use a volume name or an absolute path.

**Run** on the Images page opens this same form, with the image chosen.

Docker Hub search needs no account. Captain sends only the search text, 400 ms after you stop typing, and keeps the results until it quits. If Docker Hub answers "too many requests", Captain waits as long as Docker Hub asks before it searches again. Offline, the list shows only the engine's images. Pulls without a login are limited to 100 in 6 hours; Captain says so when a pull fails for that reason.

## Start from a template

Captain has six templates with pinned image tags:

| Template | Image | Ports | Data |
|---|---|---|---|
| PostgreSQL | `postgres:18` | 5432 | volume `data` at `/var/lib/postgresql` |
| MySQL | `mysql:8.4` | 3306 | volume `data` at `/var/lib/mysql` |
| Redis | `redis:8-alpine` | 6379 | volume `data` at `/data`, with an append-only file |
| MongoDB | `mongo:8` | 27017 | volume `data` at `/data/db` |
| RabbitMQ | `rabbitmq:4-management` | 5672, and 15672 for the web page | volume `data` at `/var/lib/rabbitmq` |
| Web server | `nginx:stable-alpine` | 8080 to 80 | the project's `site` folder, read-only |

1. Choose **Start from a template**.
2. Pick a template. Type to filter the list.
3. Check the name and the ports. Captain suggests ports that no container publishes and that nothing on this computer uses.
4. For templates with a password, Captain makes a random one. Click the eye to see it, or type your own.
5. Click **Create project**.

Every template publishes its ports on `127.0.0.1` only, so other computers on your network cannot reach the database. To share a service on purpose, change its port line in `compose.yaml`, for example from `"127.0.0.1:6379:6379"` to `"6379:6379"`.

### Passwords and .env

Passwords and user names go in a `.env` file next to `compose.yaml`. `compose.yaml` reads them with `${NAME}`, so the file has no secrets and you can commit it. Captain also writes a `.gitignore` that keeps `.env` out of Git.

On macOS and Linux, only you can read `.env` (mode 600). On Windows, Captain does not change the file's permissions: `.env` gets the permissions of its folder. The default `~/Captain` is in your user folder, which only you and administrators can read. If you set `projects_dir` to a shared folder, other users of that folder can read `.env`.

The PostgreSQL, MySQL, Redis, MongoDB, and RabbitMQ templates add tasks under `x-captain.tasks`, such as `databases` and `db-size` for PostgreSQL. In a task, `$$NAME` is a variable for the container's shell, for example `$$MYSQL_ROOT_PASSWORD`; Captain gives the shell `$NAME`. The project page runs them; see [Projects and tasks](projects-and-tasks.md). The web server template writes `site/index.html` to start with.

## Open a folder

1. Choose **Open a folder**.
2. Pick a folder, or a Compose file.

In a folder, Captain looks for `compose.yaml`, `compose.yml`, `docker-compose.yml`, and `docker-compose.yaml`, in that order, and adds an override file such as `compose.override.yaml` when there is one. Captain checks the files with `docker compose config`. If the check fails, the sheet shows Compose's error, and Captain remembers nothing. The folder stays where it is.

## Paste a docker run command

1. Choose **Paste a docker run command**.
2. Paste the command, for example from an image's README. Lines that end in `\` are joined.
3. Read the result under the field: the service and a warning for each part Captain did not convert.
4. Check the project name. Captain takes it from `--name`, or from the image.
5. Click **Create project**.

Captain converts these flags: `-p`, `-e`, `--env-file`, `-v`, `--name`, `--restart`, `--network`, `-w`, `--entrypoint`, `-u`, `-h`, `-l`, `-m`, `--cpus`, `--platform`, `--pull`, `-i`, and `-t`. Everything after the image is the command. `-d` needs nothing, because `up` runs in the background. `--rm` has no Compose key, so the container stays after it stops; Captain warns about it.

Captain leaves every other flag out and names it in a warning, for example "Captain did not convert `--privileged`. Add it to compose.yaml by hand." A bare `-e NAME` takes its value from the shell that runs `docker compose`, as `docker run` does. If a variable or label is set twice, the last value wins. A flag that `docker run` does not have is refused; write it as `--flag=value` or remove it.

Secret values go in `.env`, as for Run an image.

A relative path, such as `-v ./site:/srv` or `--env-file ./app.env`, meant the folder where you ran the command. In the project it means the project folder, so Captain keeps the path and adds a warning. Copy the files into the project folder, or write the full path in `compose.yaml`.
