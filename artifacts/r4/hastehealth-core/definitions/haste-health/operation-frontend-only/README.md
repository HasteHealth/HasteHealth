# Frontend-only operations

These OperationDefinitions are consumed by the TypeScript packages (the admin app
invokes `$invite-user` and `$deploy`) but are not implemented by the Rust server.

They live outside `../operation` on purpose: `backend/scripts/operation_build.sh`
generates `haste-generated-ops` from that directory, and the `haste-artifacts`
crate embeds it, so keeping these here leaves the backend's generated code and
its embedded resources unchanged. Move a file into `../operation` when the server
grows an implementation for it.
