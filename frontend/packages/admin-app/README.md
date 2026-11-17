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

## Themes and accents

The **Appearance** section in Settings stores two local preferences:

- `hastehealth-theme-preference` — `light`, `dark` or `system`;
- `hastehealth-theme-accent` — an accent id from [`src/theme/accents.ts`](./src/theme/accents.ts).

The default teal accent has no `data-accent` attribute. Any other accent id is
written to `data-accent` on `<html>`; the matching CSS block lives in
[`src/index.css`](./src/index.css). The pre-paint script in `index.html` applies
both preferences before the first paint, so a reload does not flash the wrong
palette.

To add an accent:

1. add an entry to `ACCENT_THEMES` in `src/theme/accents.ts` (id, label,
   description and preview swatches), and
2. add an `html[data-accent="<id>"] { ... }` block in `src/index.css` with the
   brand channels for that accent.

The components read the `--ci-*` tokens, so no component needs to change. A
deployment can also supply an accent without registering it by setting
`data-accent` and the corresponding CSS variables; the registry only drives the
picker.

## Deployment

The app is served from Cloudflare Workers as static assets ([`wrangler.jsonc`](./wrangler.jsonc),
routes on `haste-admin.app`). The GitHub workflow `<Release> Admin App`
(`.github/workflows/release_admin_app.yml`) builds and deploys it; it only runs
when started by hand from the Actions tab, with the branch or tag to build and
the two configuration values above as inputs. `pnpm wrangler` does the same from
a local checkout, using the `.env` values.

## Documentation

The user-facing guide is [docs/guides/admin_app.mdx](../website/docs/guides/admin_app.mdx)
in the website package. Update it alongside changes to routes, sidebar entries
or the Settings view.
