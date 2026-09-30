# Extensions

Captain runs Docker Desktop extensions. Each extension opens in its own window, and it talks to Captain the same way it talks to Docker Desktop.

You can install extensions on any engine, but their windows open only on macOS.

An extension runs code from its publisher on your Mac and in the engine, with your permissions. Install only extensions you trust.

## Install an extension

1. Click the Extensions button in the sidebar.
2. Enter the image, for example `docker/disk-usage-extension`.
3. Click **Install**, or press Return. Captain pulls the image.
4. Read the dialog. It shows the **Publisher**, the **Website**, whether it has a **Page**, its **Backend**, and any **Host binaries** that run on your Mac.
5. Click **Install**.

The extension's files go in `~/.captain/extensions`. A backend runs as a Compose project in the engine.

Captain lists the extensions of the engine it is connected to.

## Open an extension

Click **Open**. The extension opens in its own window. Click **Open** again to bring the window to the front.

## Update an extension

1. Click **Update…**.
2. Enter a **Tag**. The default is `latest`.
3. Click **Check**. If the image is the same, a message says the extension is up to date.
4. If a new image exists, the dialog shows both versions. Click **Update**.

The backend's volumes stay. If the update fails, Captain puts the old version back.

## Remove an extension

Click **Remove…** and confirm. Captain closes the window, removes the backend with its volumes, deletes the files, and removes the image.

## What works

Captain supports the common part of the Docker extension SDK: backend calls, `docker` and host commands, container and image lists, toasts, the file dialog, and opening links. A call that Captain does not support fails with "`<method>` is not supported by Captain". There is no marketplace. Install extensions by their image name.
