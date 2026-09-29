# Feature 0026: Docker contexts and remote hosts

- Milestone: M7 (settings and contexts)
- Status: Implemented; checked with unit tests and the live context test
- Builds on: [0008](0008-settings.md), which lists these items as out of scope
- Decision: [ADR 0004](../adr/0004-settings-file.md) (the settings file keeps the endpoint)

## Goal

The user sees the Docker CLI contexts in Settings and connects Captain to any of them. The user can make a `captain` context for Captain Engine and make a context the Docker CLI default. Captain also connects to a remote engine over SSH (`ssh://user@host`).

## In scope

- Context store parsing in `captain-core` (`docker_context`). The Docker CLI keeps each context in `<config dir>/contexts/meta/<sha256 of the name>/meta.json`, with `Name`, `Metadata.Description`, and `Endpoints.docker.Host`. `config.json` names the default in `currentContext`. The config dir is `DOCKER_CONFIG`, else `~/.docker`. Captain reads a context by its hash directory, lists all contexts sorted by name, and never writes the store itself.
- The Switch engine card lists the contexts under the detected engines. Each row shows the name, the host, and the description. The CLI default has the note "Default context". **Use** connects Captain to the context's host, the same as a detected engine. **Make default…** asks first, then runs `docker context use NAME`. The `default` context has no metadata, so it is not listed; its socket is in the detected list.
- **Create context…** writes a `captain` context that points at Captain Engine's socket. It asks first, then runs `docker context create captain --description "Captain Engine" --docker host=unix://…`. If a `captain` context exists with another host, the button says **Update context…** and runs `docker context update`. The row hides when the `captain` context already points at Captain Engine.
- Captain runs these commands with the user's config dir in `DOCKER_CONFIG`, not Captain's own (feature 0021), and without `DOCKER_HOST` and `DOCKER_CONTEXT`. Captain never removes a context.
- SSH URLs in `captain-core` (`ssh`): `ssh://[user@]host[:port][/remote/socket]`, the same form as the Docker CLI. A password, a query, or a fragment is an error. A path sets the remote socket; the default is `/var/run/docker.sock`.
- The SSH tunnel in `captain-docker` (`ssh_tunnel`). For an `ssh://` endpoint, Captain runs the system `ssh`:

  ```
  ssh -nNT -o BatchMode=yes -o ExitOnForwardFailure=yes -o StreamLocalBindUnlink=yes
      -o ConnectTimeout=30 -o ServerAliveInterval=15 -o ServerAliveCountMax=3
      -L <local socket>:<remote socket> [-l user] [-p port] -- host
  ```

  The local socket is in a new directory with mode 0700 in the temp dir. Captain waits for the socket to appear, then connects to it like any Unix socket. Compose, Buildx, and extensions use the same socket.
- The tunnel restarts when `ssh` exits, after 1 s, then with a longer wait each time, up to 30 s. It stops when Captain connects to another engine and when Captain quits.
- With `BatchMode=yes`, `ssh` never asks for a password. If the login fails, the error says: "SSH cannot log in to HOST without a password. Add your key to the SSH agent (`ssh-add`) or set up key login, then try again." An unknown host key gets its own message.
- The custom endpoint field and `DOCKER_HOST` or the current context take `ssh://` URLs. The Engine card shows the `ssh://` URL, not the local socket.

## Out of scope

- TLS settings of a context (`contexts/tls`). A `tcp://` context with TLS fails to connect.
- Contexts with a Kubernetes endpoint only; Captain shows the Docker endpoint.
- Creating other contexts, editing them, or removing them.
- SSH on Windows. The tunnel needs a Unix socket, and Captain shows an error there.
- Host binaries of extensions on a remote engine; they run on this machine.
- Passwords and passphrases in a Captain prompt. Captain relies on the SSH agent or a key without a passphrase.

## Notes

- The store format: [metadatastore.go](https://github.com/docker/cli/blob/master/cli/context/store/metadatastore.go) writes `meta/<id>/meta.json`, and [store.go](https://github.com/docker/cli/blob/master/cli/context/store/store.go) makes the id with `digest.FromString(name).Encoded()`, the SHA-256 hex digest. `currentContext` is in [configfile/file.go](https://github.com/docker/cli/blob/master/cli/config/configfile/file.go). Commands: [docker context create](https://docs.docker.com/reference/cli/docker/context/create/) and [docker context use](https://docs.docker.com/reference/cli/docker/context/use/).
- The SSH URL rules come from the CLI's [connhelper/ssh/ssh.go](https://github.com/docker/cli/blob/master/cli/connhelper/ssh/ssh.go). The CLI's [connhelper.go](https://github.com/docker/cli/blob/master/cli/connhelper/connhelper.go) runs `ssh … docker system dial-stdio` for each connection, with `-T` and `ConnectTimeout=30`, and uses the URL path as the remote socket.
- Bollard: [issue 244](https://github.com/fussybeaver/bollard/issues/244) asked for SSH. Bollard 0.21 has an `ssh` feature (`src/ssh.rs`) that starts an `openssh` master and runs `docker system dial-stdio` for each connection. That needs the `docker` CLI on the remote host, and it gives no socket to the Compose and Buildx CLIs. One forwarded socket serves bollard and the CLIs, so Captain uses a tunnel instead.
- OpenSSH forwards a local Unix socket to a remote one with `-L local_socket:remote_socket`; see [ssh(1)](https://man.openbsd.org/ssh). `BatchMode`, `ExitOnForwardFailure`, `StreamLocalBindUnlink`, and `ServerAliveInterval` are in [ssh_config(5)](https://man.openbsd.org/ssh_config).
- `ssh` makes the local socket only after it logs in, so the socket file tells Captain that the tunnel is up.
- If Captain crashes, the `ssh` process can stay. It stops by itself when the connection drops, because of `ServerAliveInterval`.

## Verification

1. Run `cargo test -p captain-core docker_context ssh`. The tests parse context metadata, check the hash directory name, and parse SSH URLs.
2. Run `cargo test -p captain-docker ssh_tunnel`. The tests run a stub `ssh`: the tunnel starts, restarts after the stub dies, stops, and reports a failed login.
3. Run `cargo test -p captain-docker --test live_contexts -- --ignored`. It creates a context and makes it the default with the real `docker` CLI in a temp `DOCKER_CONFIG`, and Captain reads the result.
4. Start Captain and open Settings. The Switch engine card lists the contexts from `docker context ls`, and the default one has the note "Default context".
5. Click **Use** on a context. The Engine card shows its host.
6. Click **Create context…** and confirm. `docker context ls` shows `captain`. The row goes away.
7. Click **Make default…** on `captain` and confirm. `docker context ls` marks `captain` with `*`.
8. Type `ssh://user@host` for a host with key login and click "Use this engine". The Engine card shows Connected and the `ssh://` URL. Kill the `ssh` process. Captain starts it again.
9. Type `ssh://user@host` for a host that needs a password. The Containers page shows the SSH key message.
10. Quit Captain. No `ssh -nNT` process stays.
