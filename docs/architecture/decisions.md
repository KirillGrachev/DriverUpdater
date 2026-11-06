# Architecture decision record

Status: accepted, 2026-11-06.

This record closes the architecture phase that followed the Driver Booster
research. It records the decisions that are now implemented, the decisions that
remain deliberately out of scope, and the one security migration that is still
open.

## Context

Driver Updater must update real Windows drivers without shipping firmware,
vendor blobs or a proprietary driver database. The earlier Driver Booster
research described the vendor model in [driver-detection.md](driver-detection.md):
a PnP inventory joined with an encrypted vendor catalog, vendor-specific install
state and a download cache. That model is useful as a reference, but its
database is not a public or legally redistributable schema.

The application now has native inventory, native Windows Update discovery,
native System Restore and native driver export. It also has a release pipeline
with Windows installers and code-scanning, dependency and secret checks.

## Decisions

### 1. Windows Update is the canonical driver source

Driver candidates come from the Windows Update Agent. A candidate is accepted
only when its hardware id matches the device hardware ids; a title match alone is
not sufficient. Versions are compared numerically, not as strings.

Rejected alternatives:

- decrypting or bundling the Driver Booster database;
- scraping vendor download pages;
- downloading arbitrary executables from a vendor CDN.

A vendor catalog may be added later only through the signed JSON contract in
[driver-detection.md](driver-detection.md). It must remain a metadata layer and
must not become an unverified executable download path.

### 2. Device discovery stays native

Inventory comes from SetupAPI with a CIM fallback. The UI receives the shared
model from `src/ipc/types.ts`; it never invents a version, size or restore point.
Device identity is the Windows device instance id, so two identical adapters are
two distinct rows.

### 3. Restore and backup stay in Windows

Restore points are created and removed through `srclient.dll` and listed or
restored through the System Restore WMI provider. Driver backups are exported and
restored with `pnputil`. The application does not implement a private driver
store or a custom rollback database.

The `srclient` entry points are resolved at runtime rather than through a static
import library, because the MSVC import library is not linkable with every
dependency combination. This is covered by a Windows unit test.

### 4. State is versioned, DPAPI-sealed and separate from caches

Settings, driver cache and installed-component state are separate versioned
files. Secrets are sealed with DPAPI and the application identifier is stable.
Every write is atomic and the repository lock is never held across an install,
restore or backup operation.

### 5. No production fixture fallback

Fixtures are test-only. A failed native call produces a typed error and an empty
or error state in the UI. The UI must not silently replace real data with sample
drivers.

### 6. The elevated UI is an accepted temporary risk

The UI still runs with the administrator token required by the installer
manifest. This is documented as C-3 in
[elevated-helper.md](elevated-helper.md) and remains the largest open security
item. Until the helper lands, the mitigations are:

- typed, allow-listed commands;
- hardware-id and installed-version verification before an install;
- validated restore-point names and package versions;
- a fixed component catalog;
- a close guard that prevents interrupting a transaction;
- structured command logging.

The target architecture is a medium-integrity UI process that talks to a signed,
separately elevated helper over a per-session named pipe. The helper exposes the
ten operations listed in [elevated-helper.md](elevated-helper.md) and accepts no
paths, shell strings or arbitrary executable names.

### 7. Release signing and auto-update require external material

Authenticode signing and auto-update signing need a certificate, a private key
and an update endpoint that cannot live in the repository. The release pipeline
is ready to consume them, but the application does not pretend that an unsigned
local build is a trusted release. `SECURITY.md` and `CODE_SIGNING.md` state the
requirements.

## Consequences

- The default product has no vendor database and no bundled driver packages.
- Update coverage is limited to drivers published through Windows Update, which
  is the trusted source Windows already knows how to install and roll back.
- The application can add a vendor catalog without changing the inventory or
  install pipeline, provided the signed metadata contract is respected.
- The C-3 helper is a prerequisite for calling the current security model
  complete. Until then, the project explicitly treats the UI as an elevated
  application.

## Follow-up work

1. Implement the C-3 helper and remove `requireAdministrator` from the UI
   manifest.
2. Configure Authenticode signing when a certificate is available.
3. Configure the auto-update endpoint and signing key when a release channel is
   available.
4. Keep the fixture-free production path covered by the native Windows tests.
