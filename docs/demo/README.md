# Demo project

`acme-shop` is a small Compose project for trying Captain. The screenshots and screen recordings in [docs/images](../images) show it.

It has five services:

| Service | What it is |
|---------|------------|
| `web` | nginx with a static page from `web/`, on `localhost:18080`. |
| `api` | A small Python server, built from `api/`, on `localhost:18000`. |
| `db` | Postgres 17 with a health check and the volume `db-data`, on `localhost:15432`. |
| `cache` | Redis 7, on `localhost:16379`. |
| `worker` | An Alpine container that runs out of memory on purpose. |

The `worker` service has a 64 MB memory limit and `restart: always`. About 8 seconds after each start, it fills its memory, and the kernel stops it with exit code 137. The service card then shows "Exit 137 · out of memory" and the **Raise memory** button, and the project log marks each crash.

The Compose file also has three [tasks](../guide/projects-and-tasks.md#tasks) under `x-captain.tasks`:

- `db-size` prints the size of the database.
- `seed` adds 500 rows to an `orders` table.
- `ping-cache` runs `redis-cli ping`.

## Run it

1. Start Captain and wait until Captain Engine runs.
2. In a terminal that uses Captain Engine, go to the project folder:

   ```sh
   cd docs/demo/acme-shop
   ```

3. Start the project:

   ```sh
   docker compose up -d --build
   ```

4. Click `acme-shop` in Captain's sidebar.

If port 18000, 18080, 15432, or 16379 is in use on your Mac, change the left number of that port in `compose.yaml`.

To remove the project and its data, run `docker compose down -v --rmi local` in the same folder.

The Postgres password in `compose.yaml` is for this demo only.
