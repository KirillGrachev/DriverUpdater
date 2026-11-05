# Code signing

Release builds are unsigned by default. The release workflow supports
Authenticode signing without storing a certificate in the repository.

## Required repository secrets

- `WINDOWS_CERTIFICATE`: base64 of a PFX file containing the code-signing
  certificate and its private key.
- `WINDOWS_CERTIFICATE_PASSWORD`: the PFX password.

Create the base64 value on a trusted machine:

```powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes("driver-updater.pfx")) |
  Set-Clipboard
```

Store it only as a GitHub Actions secret. Never commit the PFX, its password or
the base64 value.

## What the workflow does

1. Writes the PFX to the runner temporary directory.
2. Imports it into `Cert:\CurrentUser\My`.
3. Selects the imported certificate with a private key.
4. Writes a temporary Tauri config with the certificate thumbprint, SHA-256
   digest and a timestamp URL.
5. Passes that config to `tauri build`, so the application executable, NSIS
   installer and MSI are signed.
6. Leaves the certificate only on the ephemeral runner.

If `WINDOWS_CERTIFICATE` is not set, the signing step is skipped and the build
continues unsigned.

## Verification

After a signed release:

```powershell
Get-AuthenticodeSignature ".\Driver Updater_<version>_x64-setup.exe" |
  Format-List Status, StatusMessage, SignerCertificate
```

`Status` must be `Valid`. A timestamp keeps signatures valid after the
certificate expires.

## Limitations

- A self-signed certificate still triggers SmartScreen warnings unless the
  machine trusts the certificate.
- A standard OV certificate builds reputation over time. An EV certificate is
  the fastest route to SmartScreen trust but costs more.
- Signing does not replace the SHA-256 hashes published with release assets.
