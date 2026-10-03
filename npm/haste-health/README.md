# haste-health

The [Haste Health](https://haste.health) CLI and FHIR server.

## Install

```bash
npm install -g haste-health
haste-health --help
```

Or try it without installing:

```bash
npx haste-health --help
```

Builds exist for Linux (x64 and arm64) and macOS on Apple Silicon, and npm installs only the one for your machine. The Linux builds are static, so they run on any distribution.

### Windows

Run haste-health inside [WSL](https://learn.microsoft.com/windows/wsl/install):

1. In PowerShell, install WSL: `wsl --install`
2. Open the WSL terminal, install Node.js there, and run `npm install -g haste-health`.

A server started in WSL is reachable from Windows on `localhost`.

## Documentation

See the [CLI guide](https://haste.health/docs/tutorials/cli).
