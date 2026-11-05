# Elevated helper architecture

## Current state

The UI process runs with the same administrator token as the privileged
operations. This keeps IPC simple, but it means a compromised webview would
inherit the ability to run arbitrary elevated code. This is the largest
remaining security gap and is tracked as C-3.

## Target split

The target is a medium-integrity UI process and a separate elevated helper:

- the UI is started with `asInvoker`;
- the helper is started only when a privileged operation is requested;
- the helper exposes a narrow, versioned command surface;
- the UI cannot execute arbitrary commands or open arbitrary paths;
- read-only inventory, settings, restore-point listing and backup listing stay
  in the UI process.

## Helper surface

The helper accepts only these operations:

| Operation            | Input                                              |
| -------------------- | -------------------------------------------------- |
| `installDriver`      | driver id, restore point name, allow-missing flag  |
| `installUpdates`     | driver ids, restore point name, allow-missing flag |
| `createRestorePoint` | name                                               |
| `restorePoint`       | restore point id                                   |
| `deleteRestorePoint` | restore point id                                   |
| `createBackup`       | none                                               |
| `restoreBackup`      | backup id                                          |
| `deleteBackup`       | backup id                                          |
| `installPackage`     | allowed winget id, validated version               |
| `cancelPackage`      | allowed winget id                                  |

Every id is validated against the same allow-list or repository lookup as
today. The helper never accepts a path, a shell string or an arbitrary
executable name.

## Transport

- Windows named pipe created by the helper.
- Pipe name includes the current user SID and a random nonce.
- The first message carries the protocol version, nonce and caller pid.
- The helper verifies that the caller pid belongs to the same user and is the
  UI process that started it.
- Requests and responses are length-prefixed JSON with a maximum frame size.
- The helper exits after an idle timeout and when the UI process exits.

## Elevation and consent

The UI starts the helper with `ShellExecuteW` and the `runas` verb. UAC is
shown once per helper lifetime, not once per command. The helper is a separate
signed binary and has its own manifest requiring `highestAvailable`.

## Migration phases

1. Extract the privileged command implementations into a library crate shared
   by the app and the helper.
2. Add the helper binary with the whitelisted command surface and tests for
   malformed frames, oversized frames and unknown commands.
3. Start the helper from the UI and route privileged commands through it.
4. Remove administrator from the UI manifest and change the installer to
   install the helper with the same publisher identity.
5. Add integration tests that prove the UI cannot call privileged operations
   without the helper.

## Current mitigation

Until the split lands:

- privileged commands are typed and allow-listed;
- driver installs are verified by device id and installed version;
- restore-point names and package versions are validated;
- package ids come from a fixed catalog;
- cancellation never kills an active MSI/DISM transaction;
- all commands are logged;
- the release notes and SECURITY.md state that the app requires administrator
  rights and should be treated as an elevated application.
