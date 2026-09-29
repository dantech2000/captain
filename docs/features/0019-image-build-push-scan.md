# Feature 0019: Image build, push, and scan

- Milestone: M17
- Status: Done; needs a manual check in the app
- Parity: the Build and Scan actions of the Rancher Desktop Images page ([docs](https://docs.rancherdesktop.io/ui/images)), plus Tag and Push

## Goal

Build an image from a Dockerfile, tag it, push it to a registry, and scan it for vulnerabilities, without the CLI.

## In scope

- Build: a Build button in the Images toolbar opens a dialog.
  - Context: a folder, picked with the system folder dialog or typed.
  - Dockerfile: a path relative to the context. The default is `Dockerfile`. A Choose button picks a file.
  - Tag: required, for example `myapp:dev`. A missing tag means `latest`.
  - Build arguments: `KEY=value` rows, like the environment rows of the Run dialog. Empty rows are skipped.
  - Target: an optional stage name.
  - `BuildForm::to_spec` in `captain-core` checks the form. The first problem shows in red.
  - Build streams the builder output into a log view in the dialog, and follows the end. The dialog then shows "Built myapp:dev" in green, or the error line in red. The dialog stays open, so the user can read the log.
- Tag: a Tag button in the inspector opens a dialog with one field for the new reference, for example `registry.example.com/team/app:1.0`. The engine adds the tag (`POST /images/{name}/tag`). A digest reference is refused.
- Push: a Push button in the inspector opens a dialog that lists the image's tags. The user picks one. The push runs on the page, with a progress row like the pull row: status, "n of m layers", and a bar.
  - Credentials come from the Docker config (`$DOCKER_CONFIG/config.json`, else `~/.docker/config.json`), the same way the Docker CLI reads them: `credHelpers[host]`, then `credsStore`, then the `auths` entry.
  - A helper runs as `docker-credential-<name> get` with the server address on stdin. Captain looks for it on `PATH`, then in the folders where it looks for `docker`.
  - When the registry refuses the push (`denied`, `unauthorized`, `authentication required`), the error adds: "Run `docker login <host>` in a terminal, then push again." It says "Captain found no login for <host>" when the config has none.
- Scan: a Scan button in the inspector opens a results dialog.
  - Captain runs the `aquasec/trivy` image as a container, like Rancher Desktop. It pulls the image the first time. The container mounts `/var/run/docker.sock` (the engine socket inside the VM) and the named volume `captain-trivy-cache` at `/root/.cache/`, which keeps the vulnerability database between scans.
  - The command is `image --format json --scanners vuln <image>`. Trivy's log lines show as the status while it runs, for example "Downloading vulnerability DB...". Captain removes the container when the scan ends or the dialog closes.
  - `captain-core` parses the JSON report: the severity counts, and a list with the ID, package, installed and fixed versions, severity, and title.
  - The dialog shows a segmented severity filter with counts (All, Critical, High, Medium, Low, Unknown), and the list sorted by severity, then ID. An image with no findings says so.
- Engine API: `ImageApi` gets `tag_image`, `push_image` (a stream of `PullProgress`), and `scan_image` (a stream of `ScanProgress`: status lines, then the report). A separate `ImageBuilder` trait runs builds, like `ProjectRunner` runs Compose.

## Out of scope

- Registry login in Captain. The user runs `docker login`.
- Build secrets, SSH forwarding, cache options, multi-platform builds, and pushing from the Build dialog.
- Scanning for secrets, misconfigurations, and licenses. Only vulnerabilities.
- A details panel with references for each vulnerability. The list shows the title.
- Scans on a remote engine that has no `/var/run/docker.sock`.

## Notes

### Build: the `docker buildx build` CLI, not `POST /build`

The Engine API has `POST /build`, and bollard has `build_image`. We do not use it:

- The request body is a tar of the context. Captain would have to build the tar and apply `.dockerignore` with the same rules as the CLI.
- Without BuildKit, the engine uses the legacy builder, which Docker deprecated ([deprecated features](https://docs.docker.com/engine/deprecated/#legacy-builder-for-linux-images)). BuildKit through the API needs a gRPC session over `/session` for the context and secrets. Bollard supports it only behind its `buildkit` feature, with extra protobuf dependencies ([bollard README](https://github.com/fussybeaver/bollard#buildkit), [`build_image`](https://docs.rs/bollard/latest/bollard/struct.Docker.html#method.build_image)).
- The CLI already does all of this, and it builds exactly as `docker build` does in a terminal. This is the same trade as Compose in [ADR 0005](../adr/0005-compose-via-cli.md).

So `captain-docker` has a `BuildCli`. It finds `docker` with the Compose locator and checks `docker buildx version` once per connection. It runs:

```
docker buildx build --builder default --progress plain --load -t <tag> -f <dockerfile> [--build-arg K=V]... [--target T] <context>
```

- `DOCKER_HOST` is Captain's endpoint and `DOCKER_CONTEXT` is removed, as for Compose. With `DOCKER_HOST` set, the `default` builder is the `docker` driver on that engine, so the image lands there. `--builder default` ignores a builder the user picked with `docker buildx use`. `--load` is a no-op with that driver, and it keeps the image local with any other driver.
- `--progress plain` writes one line per step on stderr ([buildx build](https://docs.docker.com/reference/cli/docker/buildx/build/#progress)). Captain streams each line.
- A failed build reports its `ERROR:` line.
- Without the buildx plugin, the Build button is off, with a tooltip: "Install Docker Buildx to build images."

### Push credentials

- The engine wants `X-Registry-Auth`, base64 JSON with `username`, `password`, and `serveraddress`, or an `identitytoken` ([Engine API authentication](https://docs.docker.com/reference/api/engine/version/v1.47/#section/Authentication)). Bollard sends a `DockerCredentials` in that header ([`push_image`](https://docs.rs/bollard/latest/bollard/struct.Docker.html#method.push_image)).
- The CLI picks the store for a host: `credHelpers[host]`, else `credsStore`, else the file ([config file](https://docs.docker.com/reference/cli/docker/login/#credential-stores), [`configfile/file.go`](https://github.com/docker/cli/blob/master/cli/config/configfile/file.go)). Docker Hub's server address is `https://index.docker.io/v1/`. File entries hold `auth`, base64 of `user:password`, and match by host name, without scheme or path.
- A helper prints `{"ServerURL","Username","Secret"}` ([credential helpers](https://github.com/docker/docker-credential-helpers)). The user name `<token>` means the secret is an identity token ([`native_store.go`](https://github.com/docker/cli/blob/master/cli/config/credentials/native_store.go)). "credentials not found" means no login.
- Bollard's `PushImageInfo` has no layer ID, so push progress counts `Preparing` messages as layers and `Pushed`, `Layer already exists`, and `Mounted from …` as finished layers.

### Scan with Trivy

- Trivy's container docs mount the Docker socket and a cache folder at `/root/.cache/` ([installation](https://trivy.dev/latest/getting-started/installation/#use-container-image)).
- The JSON report has `SchemaVersion` 2, `ArtifactName`, and `Results`. Each result has a `Target`, a `Class`, and `Vulnerabilities`, which may be `null`. Each vulnerability always has `VulnerabilityID`, `PkgName`, `InstalledVersion`, and `Severity`; `FixedVersion`, `Title`, and `PrimaryURL` may be missing ([reporting](https://trivy.dev/latest/docs/configuration/reporting/#json)).
- Trivy 0.74 downloads a 118 MB database, which unpacks to about 1.4 GB in `captain-trivy-cache`. The first scan takes a few seconds more; later scans reuse it.

## Verification

1. Build a folder with `FROM busybox` and `ARG MSG` / `RUN echo $MSG > /msg`, tag `captain-test:dev`, and the argument `MSG=hi`. The log streams, the dialog says "Built captain-test:dev", and the image appears in the list.
2. A build with a Dockerfile name that does not exist shows the `ERROR:` line in red.
3. An empty tag, a tag with a space, and a build argument without `=` show a red hint and do not start a build.
4. Tag `captain-test:dev` as `localhost:5000/captain-test:dev`. The inspector lists both tags.
5. With `docker run -d -p 5000:5000 --name captain-registry registry:2`, Push `localhost:5000/captain-test:dev`. The progress row reaches 100%.
6. Push `captain-test:dev` to Docker Hub without a login. The error says "Run `docker login docker.io`…".
7. Scan `debian:bookworm-slim`. The first scan shows the database download, then counts by severity. The filter shows only the chosen severity. `docker ps -a` shows no Trivy container afterwards.
8. `cargo test -p captain-docker --test live_build -- --ignored` builds `captain-agent-build:test` from a temp folder, tags it, and removes both tags.
