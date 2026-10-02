# Moving from Docker Desktop or Rancher Desktop

This page moves your containers, your terminal, and your Kubernetes contexts to Captain. Do the steps in order. Uninstall the old app last.

The Migration Assistant and the terminal setup need macOS.

## What Captain never does

- Captain never deletes or changes anything in the old engine. The one exception is a switch-over, which stops the containers you confirm. It never removes them.
- Captain never overwrites an item that already exists in Captain Engine. It skips the item and says why.

So you can copy, test, and go back at any time.

## Copy your data

The Migration Assistant copies networks, volumes, images, Compose projects, and containers from another engine into Captain Engine. It uses the Docker API of both engines.

### Open the Migration Assistant

1. Start the old app, so that its engine runs.
2. Open the Migration Assistant in one of two ways:
   - On the first launch, check **Bring your data along** on the setup screen.
   - Press ⌘K and choose **Bring data from another engine…**.
   - In Settings, open the **…** menu of the Engine section and choose **Bring data from another engine…**.

### Step 1: Source

1. Under **Copy from**, pick the old engine. Captain names the engines it knows, such as Rancher Desktop or Docker Desktop.
2. If the engine is not in the list, choose **Other engine** and enter its address, for example `unix:///path/to/docker.sock`.

If the list says **No other engines found**, start the old app, then open the assistant again.

### Step 2: Plan

The plan lists every item, grouped as networks, volumes, images, Compose projects, and containers. Everything starts checked.

1. Uncheck the items that you do not want.
2. Choose **All images** or **Only in use**. **Only in use** skips images that no container uses.
3. Read the size and time estimate. If the target has too little free space, the plan says so and you cannot start. Uncheck items, or give Captain Engine a larger disk in [Settings](settings.md).
4. Look for containers marked **Changes inside**. They wrote files outside their volumes, and a copy loses those files. To keep them, turn on **Snapshot** for the container.
5. For a running database or service, decide about **Switch over**. See [Switch over a running item](#switch-over-a-running-item).
6. Click **Copy N items**.

### Step 3: Copy

The items copy one at a time. Each row shows its state, the bytes copied, and a note.

- If a row fails, click **Retry**.
- Click **Stop** to end the current item. Captain removes its partial copy. Click **Resume** to go on with the items that are not done.

### Step 4: Summary

The summary counts the items that copied, were skipped, or failed. It ends with "Your old engine was not changed."

- Click **Retry failed** to try the failed items again.
- Click **Done** to close the assistant.

### How items copy

- **Volumes** keep their files, owners, and modes. A volume bound to a host folder gets created empty, and its row says why.
- **Images** keep their tags.
- **Compose projects** start again with `docker compose up`, if the project folder and its Compose files are on this Mac. Otherwise Captain creates the containers one by one. The project still groups in the sidebar.
- **Containers** keep their image, command, environment, ports, mounts, labels, restart policy, and networks. A stopped container is created, but not started.

## Switch over a running item

A plain copy of a running database can catch it in the middle of a write. The old engine also keeps holding the published ports. A switch-over fixes both.

For a running project or container, turn on **Switch over** in the plan. When you click **Copy N items**, Captain asks **Stop these in the old engine?** and lists each container it will stop. Click **Stop and switch over**.

For each item, Captain then:

1. stops it in the old engine, and does not delete it,
2. copies its volumes again,
3. starts it in Captain Engine,
4. checks that it runs, that its health check passes, and that its ports answer.

The row shows each step and the downtime. The downtime is usually a few seconds.

Captain refuses a switch-over, and the row says why, in these cases:

- The container was started with `--rm`. The old engine would delete it when it stops.
- Another running container in the old engine writes to the same volume.
- Captain Engine already has a volume or container with that name that Captain did not copy.

If a step fails, click **Retry** or **Roll back**. **Roll back** stops the copy in Captain Engine and starts the original in the old engine again. The summary lists each switched item under **Switched over**, with **Roll back**.

## Set up your terminal

Rancher Desktop puts `docker`, `kubectl`, and other tools in `~/.rd/bin`. Docker Desktop puts them in `/usr/local/bin`. After you uninstall the old app, those tools are gone. Captain ships its own `docker`, Compose, Buildx, the keychain credential helper, `kubectl`, `helm`, and `captain`.

Captain changes no shell file and no docker file until you ask.

1. Open Settings. The **Terminal** row says where your `docker` comes from now.
2. Click **Set up…**. The sheet **Use Captain from your terminal** opens with three steps.
3. **Link the tools.** Captain links the tools into `~/.captain/bin` and the Compose and Buildx plugins into `~/.captain/cli-plugins`. It adds `~/.captain/cli-plugins` to `cliPluginsExtraDirs` in `~/.docker/config.json`, so `docker compose` and `docker buildx` find Captain's plugins first. Before the first change, Captain saves a copy as `~/.docker/config.json.captain-backup`.
4. **Put ~/.captain/bin first on your PATH.** See [PATH](#path) below.
5. **Make docker use Captain Engine.** Click **Use Captain Engine**. Captain creates the `captain-engine` docker context if it does not exist, and makes it the default.
6. If you want, link `/var/run/docker.sock` to Captain Engine. See [The Docker socket](#the-docker-socket).
7. Open a new terminal. Click **Check again**. The list **Where each tool comes from now** should say Captain for each tool.
8. Click **Done**.

Check the result in the new terminal:

```sh
docker context ls        # captain-engine has a *
docker ps
docker compose version
docker buildx version
```

The links point into `Captain.app`. When you move the app, Captain fixes the links at its next start.

You can do the same from the terminal with `captain tools install`. Quit Captain first. See [The command line](cli.md).

### PATH

`~/.captain/bin` must come before `~/.rd/bin` and `/usr/local/bin` on your `PATH`.

Captain can add this block to the end of `~/.zshrc`, `~/.bash_profile` or `~/.bashrc`, and fish's `conf.d/captain.fish`:

```sh
# >>> captain >>>
# Added by Captain. Set PATH to Manual in Captain's Settings to remove it.
export PATH="$HOME/.captain/bin:$PATH"
# <<< captain <<<
```

Captain writes the block only when `command_line_tools.path` is `"automatic"` in the [settings file](settings.md). The default is `"manual"`. Then the sheet shows the line, and you add it.

Captain never writes a shell file that is a link or that a tool manages. The sheet then shows what to add by hand.

- **home-manager.** home-manager links `~/.zshrc` into `/nix/store`. Add this line to your home-manager configuration, then run `home-manager switch`:

  ```nix
  home.sessionPath = [ "$HOME/.captain/bin" ];
  ```

  See the [`home.sessionPath` option](https://nix-community.github.io/home-manager/options.xhtml#opt-home.sessionPath).
- **chezmoi.** Add the `export PATH` line to the source file (`chezmoi edit ~/.zshrc`), then run `chezmoi apply`.

If a shell file sets `DOCKER_HOST` or `DOCKER_CONTEXT`, that setting wins over the default context. Remove it.

### The Docker socket

Some tools do not read docker contexts. They connect to `/var/run/docker.sock` only. Rancher Desktop and Docker Desktop link that path to their own engine.

The **Set up…** sheet offers to link `/var/run/docker.sock` to Captain Engine. macOS asks for an administrator password. Skip this step if all your tools read `DOCKER_HOST` or the docker context.

## Kubernetes contexts

When you turn on Kubernetes in Captain, Captain adds a context named `captain-desktop` to `~/.kube/config`. Before each change, it saves a copy as `~/.kube/config.captain-backup`. The other contexts stay. See [Kubernetes](kubernetes.md).

If another context is the current one, `captain` does not replace it. To switch, do one of these:

- Right-click the menu bar icon, open **Kubernetes Contexts**, and pick `captain`.
- Run `kubectl config use-context captain-desktop`.

Rancher Desktop's context is `rancher-desktop`. Docker Desktop's is `docker-desktop`. They stop working when you uninstall the old app. Delete them with `kubectl config delete-context <name>`.

Rancher Desktop's `kubectl` and `helm` in `~/.rd/bin` go away with it. Captain links its own copies into `~/.captain/bin` when you [set up your terminal](#set-up-your-terminal).

## Before you uninstall the old app

1. Copy your data with the Migration Assistant. Switch over the running databases. The old app's uninstall deletes its engine and everything in it.
2. Start your projects in Captain and check that they work.
3. Set up your terminal. Check that **Where each tool comes from now** shows Captain for `docker`.
4. Check that **Where each tool comes from now** shows Captain for `kubectl` and `helm`, if you use them.
5. If `/var/run/docker.sock` points at the old engine, link it to Captain Engine, or check that your tools do not need it.
6. Turn off the old app's start at login, and quit it.
7. Open a new terminal and run `docker ps` and `docker compose ls`. Both must list Captain Engine's containers.

Then uninstall the old app:

- **Rancher Desktop.** Follow [Uninstalling Rancher Desktop on macOS](https://docs.rancherdesktop.io/getting-started/installation/). If your shell file still has the lines between `### MANAGED BY RANCHER DESKTOP START (DO NOT EDIT)` and its end marker, delete them. Old links in `~/.docker/cli-plugins` that point into `~/.rd/bin` do no harm. The docker CLI skips broken plugin links.
- **Docker Desktop.** Follow [Uninstall Docker Desktop](https://docs.docker.com/desktop/uninstall/). Its manual cleanup step deletes `~/.docker`. That folder now holds the `captain-engine` context and Captain's plugin folder setting. If you delete it, open Settings and click **Set up…** again.

## Go back

The old engine still has your data, unless you uninstalled the old app. Items that you switched over are stopped there, not deleted.

1. Start the old app.
2. For a switched item, click **Roll back** in the Migration Assistant's summary. If the assistant is closed, start the stopped containers in the old app.
3. Make docker use the old engine again: `docker context use rancher-desktop`, or `docker context use desktop-linux` for Docker Desktop.
4. Quit Captain. Run `captain tools uninstall` to remove the links, the plugin folder setting, and Captain's PATH blocks. With home-manager, also remove the `home.sessionPath` line.
5. Switch `kubectl` back: `kubectl config use-context rancher-desktop` or `docker-desktop`.
6. If you linked `/var/run/docker.sock` to Captain Engine, link it back from the old app's settings.

To remove Captain as well, quit Captain, then delete `Captain.app`, `~/.captain`, `~/Library/Application Support/Captain`, and `~/Library/Logs/Captain`. `~/.captain` holds Captain Engine and its snapshots.
