# Driver detection and the Driver Booster model

This document records the interoperability research behind our driver pipeline.
It describes observable structures and behaviour only. It does not copy a
proprietary database, decrypted payloads or vendor binaries.

## Research scope

The analysis used a locally installed Driver Booster `13.1.0.171` with database
`2026.1102.1357`. The application was observed read-only. Its encrypted scan
payloads were not decrypted or redistributed.

Observed artefacts:

| Path                                       | Observed role                                                   |
| ------------------------------------------ | --------------------------------------------------------------- |
| `Database\sMarUpdateInfo.dbd`              | Encrypted driver metadata index, about 1.2 MB                   |
| `Database\Scan\initial.wlst`               | Encrypted initial scan payload, about 1.9 MB                    |
| `scan.dat`, `dev-pro.dat`, `cache-pro.dat` | Encrypted scan state and device cache                           |
| `Config.ini [flag]`                        | Per-driver install state keyed by hardware id, date and version |
| `Config.ini [UptodateOrder]`               | Device instance ids mapped to a packed date value               |
| `Config.ini [F2P]`                         | Drivers offered to the user, keyed like the update order        |
| `Config.ini [OutlinkSize]`                 | Download size cache keyed by an opaque content id               |
| `LocalData\RqstFilter.ini`                 | Hardware id inclusion and exclusion rules                       |
| `LocalData\Vendor.ini`                     | OEM name normalisation                                          |
| `LocalData\InstConf.ini`                   | Important device classes and Windows error ignore list          |

The useful information is in the shape of keys, matching rules and state
transitions. The encrypted blobs are not a public schema and are not a source we
can legally bundle.

## Detection pipeline observed

The behaviour is consistent with this pipeline:

1. Enumerate Plug and Play devices through Windows setup APIs, not by scanning
   the registry for INF files alone.
2. Collect the device instance id, hardware ids, compatible ids, class,
   manufacturer, driver version, driver date and INF name.
3. Normalise hardware ids to upper case and split the instance suffix from the
   hardware id.
4. Drop devices that cannot receive a meaningful update. The filter file calls
   these blacklist whole-string rules and blacklist start-with rules.
5. Normalise OEM names so one vendor row matches many spelling variants.
6. Join the device against a vendor database by hardware id, then compare the
   installed driver date and numeric version with the catalog row.
7. Rank candidates by date and version, with per-device overrides, and record
   the result in a local state file.
8. Keep a download-size cache and a list of devices already confirmed current.

### Hardware id filters

`RqstFilter.ini` contains three kinds of rule:

```ini
[BLACK_WS]
ACPIAPIC=1
ACPI\FIXEDBUTTON=1
DETECTEDINTERNAL\ACPI_HAL=1
DETECTED\ACPI_HAL=1
GENCDROM=1
GENDISK=1
UMB\UMBUS=1

[BLACK_SW]
USBSTOR\=1
HID_DEVICE=1

[MATCHING]
PCI=1
HDAUDIO=1
USB=1
HID=1
SCSI=1
```

The names mean:

- `BLACK_WS`: reject when the normalised hardware id matches the whole string.
- `BLACK_SW`: reject when it starts with the value.
- `MATCHING`: bus families that are eligible for catalog matching.

The current app uses the same device classes in `is_ignored_device`, but keeps
the policy in Rust rather than an external mutable INI file.

### OEM normalisation

`Vendor.ini` maps product and manufacturer strings to a stable vendor label:

```ini
[Map]
HP=HP
LENOVO=LENOVO
DELL=DELL
* Hewlett?Packard *=HP
```

The important idea is not the exact list. It is that vendor matching must be
normalised before catalog lookup, otherwise `Hewlett-Packard`, `HP` and
`Hewlett Packard` become different vendors.

### Driver state keys

`Config.ini [flag]` uses this key shape:

```text
<hardware-id>|<driver-date>|<driver-version>=1
```

This is not just an "installed" flag. It records the exact date and version that
was seen, so a newer catalog row for the same hardware id can be offered on the
next scan. The value is a state marker, not a boolean that permanently hides the
device.

`[UptodateOrder]` maps a full device instance id to a compressed date. It is a
fast path for devices already checked, not a substitute for comparing the
installed driver against the catalog.

`[F2P]` and the update order use the same device keys. This suggests separate
lists for "offered" and "up to date" states, both scoped to the device instance.

### Download metadata

`[OutlinkSize]` maps an opaque content hash to a byte count. The app does not
store the URL in plain text in this file. The download URL and file hash are
resolved from the encrypted index when a candidate is selected.

Our implementation gets the size directly from the Windows Update result through
`MaxDownloadSize`, so it does not need a separate size cache.

## Differences from Driver Updater

| Concern         | Driver Booster                           | Driver Updater                                      |
| --------------- | ---------------------------------------- | --------------------------------------------------- |
| Catalog         | Vendor-maintained encrypted database     | Windows Update driver catalog                       |
| Discovery       | PnP inventory joined with vendor index   | PnP inventory joined with Windows Update results    |
| Matching        | Hardware id precedence plus vendor rules | Hardware id equality and numeric version comparison |
| Update source   | Vendor packages and mirrors              | Windows Update Agent                                |
| Ranking         | Vendor score, date and version           | Version, then date and recommendation flag          |
| State           | INI flags and opaque cache files         | Versioned encrypted JSON state                      |
| Restore         | Built-in backup and restore points       | Windows System Restore plus `pnputil` export        |
| Ads and bundles | Present in the consumer product          | Not part of this application                        |

The architectural decision is deliberate: Windows Update is the canonical source
because it provides signed driver packages, rollback metadata and an install API
that Windows already trusts. A vendor database can be added later as a signed
metadata layer, but it must not become an unverified executable download path.

## Data model used by Driver Updater

The shared model mirrors `src/ipc/types.ts`:

```text
Driver
  id                 stable SHA-256 of the device instance id
  name               device name
  vendor             normalised manufacturer
  category           video, audio, chipset, network, input
  deviceId           Windows device instance id
  hardwareIds        hardware and compatible ids
  infName            published INF name
  current            installed driver version
  available          candidate driver version
  status             current, update, optional
  updateId           Windows Update identity
  sizeMb             MaxDownloadSize in MB
  recommended        Windows Update recommendation flag
  rebootRequired     result of the install
```

Rules:

- `id` is deterministic and survives a rescan while the device instance id is
  unchanged.
- A device is deduplicated by instance id, not by display name. Two identical
  adapters are distinct rows.
- Versions are compared as numeric components, not as strings. `1.0.10` is newer
  than `1.0.9`.
- A candidate is accepted only when the Windows Update hardware id matches the
  device hardware ids. A title match alone is not enough.
- Pseudo devices and disk or optical bus nodes are filtered before they reach the
  UI.
- The UI never invents a version, size or restore point. Empty states are real.

## Optional vendor metadata contract

If a vendor catalog is added, use a signed JSON document rather than an opaque
binary. Proposed schema:

```json
{
    "schemaVersion": 1,
    "generatedAt": "2026-11-05T00:00:00Z",
    "entries": [
        {
            "hardwareIds": ["PCI\\VEN_10DE&DEV_2F04"],
            "vendor": "NVIDIA",
            "name": "NVIDIA Graphics Driver",
            "version": "32.0.16.1047",
            "driverDate": "2026-05-19",
            "sizeBytes": 968052256,
            "downloadUrl": "https://vendor.example/driver.exe",
            "sha256": "hex digest",
            "signature": "base64 signature"
        }
    ]
}
```

Requirements:

- HTTPS only, with a pinned public key for the catalog signature.
- SHA-256 verification of every downloaded package.
- Authenticode verification before execution.
- A local monotonic catalog version to reject rollback.
- Explicit user consent before a vendor package is downloaded.
- No silent install of bundled software.
- A restore point and driver export before every install when enabled.

## Decision summary

- Keep Windows Update as the default and currently only update source.
- Keep device discovery in the native Windows layer.
- Keep hardware-id matching and numeric version comparison in Rust.
- Keep all user-facing text in the frontend locale files.
- Do not decrypt or redistribute the Driver Booster database.
- Add vendor catalogs only through the signed metadata contract above.

This closes the research phase. A vendor database is now an optional extension,
not a prerequisite for a working, honest driver updater.

## Implementation status

The production paths are now real Windows implementations:

- inventory comes from the native Windows provider;
- driver candidates come from Windows Update Agent;
- restore points use System Restore through `srclient` and WMI;
- driver backups use `pnputil`;
- state and driver cache are separate, versioned and DPAPI-sealed;
- UI data is never backed by production fixtures;
- install, restore and backup operations run outside the repository lock;
- long operations are guarded against closing the window;
- install progress exposes restore, backup and install stages;
- typed command errors carry stable codes;
- runtime logs are written to the application log directory.

Known limitations are tracked explicitly:

- the UI still shares the elevated token, so the medium-integrity helper is
  still the main security architecture task;
- package cancellation cannot distinguish download from install with the
  current winget CLI, so an active MSI transaction is never cancelled;
- driver inventory is native SetupAPI with a CIM fallback, while Windows Update,
  System Restore and DISM still go through their current providers;
- 33 locales still fall back to English for some older keys, guarded by a
  baseline check so the gap cannot grow silently;
- release signing is optional until a certificate is configured.
