# Feature 0015: App behavior

- Milestone: M14
- Status: Done on macOS; checked by hand (M14). Since M30, "Start in the background" is the `start_in_background` key of the settings file, and the socket link is a step of the terminal setup sheet ([0037](0037-settings-page.md)).

## Goal

Match Rancher Desktop's Application > Behavior and General preferences ([Behavior](https://docs.rancherdesktop.io/ui/preferences/application/behavior), [General](https://docs.rancherdesktop.io/ui/preferences/application/general)): Captain can start at login, start with only the menu bar icon, hide the menu bar icon, and link the default Docker socket to Captain Engine.

## In scope

- A **Behavior** card in Settings:
  - **Start at login** (macOS, Linux, Windows). The switch reads the login item from the system each time the page draws, so it shows the real state. A change the user makes outside Captain shows up.
  - **Start in the background** (macOS, Windows). Captain launches with no main window, only the menu bar icon. The login item launches the same program, so a login start follows this setting too. A click on the Dock icon, or Open Captain in the menu, opens the window.
  - **Show the menu bar icon** (macOS; "notification area icon" on Windows). Off removes the icon at once, and on puts it back. With the icon off, closing the window quits Captain, and Captain always opens its window at launch.
- An **Administrative access** card (macOS and Linux), shown while Captain Engine is the chosen engine and its socket is not already `/var/run/docker.sock`:
  - The current state of `/var/run/docker.sock`: missing, linked to Captain Engine, linked to another path (another engine), a real socket, or something else.
  - **Link to Captain Engine…** runs `ln -sfn <Captain Engine socket> /var/run/docker.sock` as root. **Remove link…** removes the link when it points at Captain Engine.
  - The root command checks the socket again right before it acts, because another engine can change it while the password prompt is open. Remove deletes the socket only if it is still a link to Captain Engine's socket. Link replaces it only if it is still what the user saw and confirmed: missing, the same other link, or a socket. Otherwise nothing changes and the card shows "/var/run/docker.sock changed after Captain read it, so Captain left it. Try again."
  - If the path is a real socket or links to another engine, a dialog says so and asks before Captain replaces it.
  - If the path is a folder or a plain file, Captain does not touch it and says why.
- Settings: `start_in_background` (off by default) and `show_menu_bar_icon` (on by default) in the settings file ([ADR 0004](../adr/0004-settings-file.md)). Old files load with these defaults.

## Out of scope

- Hiding the Dock icon in the background. GPUI sets the regular activation policy at launch and has no API to change it, and the workspace forbids `unsafe` code.
- A second launch on Linux or Windows while Captain runs. It starts a second copy; macOS sends the reopen event instead.
- Linking the socket again after each reboot. On Linux `/var/run` is a `tmpfs`, so the link is gone after a restart; the card then shows "missing" and the button works again. Rancher Desktop asks for the password after each reboot for the same reason.
- Administrative access on Windows. Windows has no `/var/run/docker.sock`.
- A Linux Captain Engine. On Linux, Captain Engine is the system `dockerd`, whose socket already is `/var/run/docker.sock`, so the card stays hidden there until Captain runs its own engine.
- Rancher Desktop's other General settings: language, automatic updates, and statistics.

## Notes

- **The macOS login item is a LaunchAgent, not `SMAppService`.** `SMAppService.mainApp` needs macOS 13 and a bundle signed with a real identity; an ad-hoc or unsigned bundle fails with "Operation not permitted" ([Apple forum 707482](https://developer.apple.com/forums/thread/707482), [push-to-talk#12](https://github.com/mgosal/push-to-talk/issues/12)). Captain.app from `scripts/bundle-macos.sh` is unsigned, and `cargo run` makes no bundle at all. Calling the Objective-C API also needs `objc2` bindings, and the workspace forbids `unsafe`. A LaunchAgent works for any binary, needs no new dependency, and reads back as a file ([launchd.plist(5)](https://keith.github.io/xcode-man-pages/launchd.plist.5.html), [Creating Launch Daemons and Agents](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html)). Captain writes `~/Library/LaunchAgents/dev.captain.Captain.plist` with `ProgramArguments` set to the running binary, `RunAtLoad`, `ProcessType` `Interactive`, and `LimitLoadToSessionType` `Aqua`. Captain does not load the agent now; launchd loads it at the next login. macOS 13 and later tells the user about a new launch agent and lists it in System Settings > General > Login Items ([Apple: Manage login items and background tasks](https://support.apple.com/guide/deployment/manage-login-items-background-tasks-mac-depdca572563/web)). Once Captain is signed (M9), a later change can move to `SMAppService`.
- **Linux** writes `~/.config/autostart/dev.captain.Captain.desktop` (`$XDG_CONFIG_HOME/autostart`) ([Desktop Application Autostart](https://specifications.freedesktop.org/autostart/latest/)). A file with `Hidden=true` or `X-GNOME-Autostart-enabled=false` counts as off, so a change in the desktop's startup settings shows. The `Exec` value quotes the path by the [Desktop Entry rules](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html): reserved characters put the argument in double quotes, `"`, `` ` ``, `$`, and `\` get a backslash, and every backslash is then doubled for the string escape. The string escape also writes a line break or tab as `\n`, `\r`, or `\t`, and a literal `%` becomes `%%`, so a path such as `/home/me/100%/captain` is not read as a field code.
- **Windows** writes the value `Captain` = `"<path to captain.exe>"` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` with the `winreg` crate ([Run and RunOnce registry keys](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)). The Task Manager's Startup switch lives in an undocumented `StartupApproved` key, so Captain does not read it.
- **The file and registry text is plain data** in `captain_core::behavior::login_item`, with unit tests. The file and registry access is in `crates/captain-app/src/login_item/`, one file per platform.
- **Administrative access** asks for the password with the system prompt. On macOS it runs `osascript` with `do shell script … with administrator privileges` ([TN2065](https://developer.apple.com/library/archive/technotes/tn2065/_index.html)). Both platforms run one fixed `/bin/sh -c` script as root. The action, the paths, and the expected state are arguments of the script, never script text. On macOS they go in as arguments of an `on run argv` handler, and `quoted form of` quotes each one for the shell. On Linux it runs `pkexec /bin/sh -c <script> sh …` ([pkexec(1)](https://www.freedesktop.org/software/polkit/docs/latest/pkexec.1.html)). The script checks the state with `test -L`, `test -S`, and `readlink` ([POSIX test](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/test.html)), and a relative link target is read from the link's folder. Cancel in the prompt shows "You canceled the request." Captain never runs these commands as root in tests; the state decisions and the command lines are plain data in `captain_core::behavior::docker_socket`, with unit tests. One test runs the script without root on a socket path in a temporary folder.
- Rancher Desktop does the same: with administrative access it puts the socket at `/var/run/docker.sock`, and without it only at `~/.rd/docker.sock` ([General](https://docs.rancherdesktop.io/ui/preferences/application/general)).
- The UI reaches the system through the `SystemIntegration` trait in `captain-ui`, which `captain-app` implements, the same way as `EngineSource`.
- The tray follows the setting: the app observes the settings global, and starts or removes the tray when `show_menu_bar_icon` changes ([ADR 0006](../adr/0006-menu-bar.md)).

## Verification

1. Build `Captain.app` with `scripts/bundle-macos.sh` and open it.
2. In Settings > Behavior, turn on Start at login. Check that `~/Library/LaunchAgents/dev.captain.Captain.plist` exists and that System Settings lists Captain under Login Items.
3. Delete the plist in Finder, and open Settings again. The switch is off.
4. Turn on Start in the background, quit, and open Captain. Only the menu bar icon appears. Click the Dock icon; the window opens.
5. Turn on Start at login, log out, and log in. Captain starts with only the menu bar icon.
6. Turn off Show the menu bar icon. The icon goes away. Close the window; Captain quits.
7. With Captain Engine running, click Link to Captain Engine… and enter the password. The card says the socket points at Captain Engine, and `docker -H unix:///var/run/docker.sock ps` lists Captain's containers.
8. Point `/var/run/docker.sock` at another path, and click Link again. A dialog names the other path and asks first.
9. Click Link and cancel the password prompt. The card says the request was canceled.
10. Click Link. While the password prompt is open, run `sudo ln -sfn /tmp/other.sock /var/run/docker.sock` in a terminal. Enter the password. The card says the socket changed, and the link still points at `/tmp/other.sock`.
11. On Linux, check steps 2 and 3 with `~/.config/autostart/dev.captain.Captain.desktop`. On Windows, check the `Run` key with `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v Captain`.
