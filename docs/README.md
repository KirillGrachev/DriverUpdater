# Documentation

This directory contains the project documentation and the source of the GitHub
wiki.

- [Project README](../README.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Third-party notices](THIRD_PARTY_NOTICES.md)
- [Wiki source](wiki/Home.md)
- [Screenshots](screenshots)

## Wiki

The wiki pages are kept in [wiki](wiki) as normal Markdown so they render in
the repository and can be reviewed like any other document. Publish them with:

```powershell
pwsh -File scripts/publish-wiki.ps1
```

The first wiki page still has to be created once through the GitHub UI; GitHub
does not expose a REST endpoint for it.
