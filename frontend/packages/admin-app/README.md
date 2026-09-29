# @haste-health/admin-app

The browser UI for a Haste Health tenant, built with Vite and React Router.

## Scripts

Run these from this directory:

| Script         | What it does                                                           |
| -------------- | ---------------------------------------------------------------------- |
| `pnpm dev`     | Dev server on port 3001 (`strictPort`, so it fails rather than moving) |
| `pnpm build`   | Production build into `dist/`                                          |
| `pnpm preview` | Serve the build locally                                                |

The components it renders come from `@haste-health/components`, so build that
package first when working across both:

```bash
pnpm --filter @haste-health/components build
```

## Configuration

[`src/config.ts`](./src/config.ts) reads `VITE_FHIR_BASE_URL` and
`VITE_CLIENT_ID`, from `import.meta.env` at build time or from `window` at
runtime (which is how the Docker image is configured without a rebuild). The
client id defaults to `admin-app`, the OIDC client the server registers for this
app; its redirect target is the server's `admin_app_redirect_uri` setting.

## Documentation

The user-facing guide is [docs/guides/admin_app.mdx](../website/docs/guides/admin_app.mdx)
in the website package. Update it alongside changes to routes, sidebar entries
or the Settings view.
