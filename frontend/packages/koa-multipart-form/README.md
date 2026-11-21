# @haste-health/koa-multipart-form

Koa middleware for `multipart/form-data` requests, built on [multer](https://github.com/expressjs/multer). It is a port of [@koa/multer](https://github.com/koajs/multer) that loads cleanly as an ES module (the original conflicts with ESM loaders such as tsx). The middleware only engages when the request's `Content-Type` is `multipart/form-data`; every other request passes straight to `next()`.

It has no FHIR dependency and works in any Koa 2 application.

## Install

```bash
npm install @haste-health/koa-multipart-form koa
```

## Usage

```ts
import Koa from "koa";
import Router from "@koa/router";
import { multer } from "@haste-health/koa-multipart-form";

const upload = multer({
  storage: multer.memoryStorage(),
  limits: { fileSize: 10 * 1024 * 1024 },
});

const router = new Router();

router.post("/upload", upload.single("file"), (ctx) => {
  // Text fields are on ctx.request.body, the file on ctx.file (also ctx.request.file).
  const { originalname, size, buffer } = ctx.file;
  ctx.body = { name: originalname, size, fields: ctx.request.body };
});

const app = new Koa();
app.use(router.routes());
```

`multer(options)` takes multer's options (`storage`, `limits`, `fileFilter`, ...) and returns an instance whose methods produce Koa middleware:

| Method                         | Accepts                              | Result                              |
| ------------------------------ | ------------------------------------ | ----------------------------------- |
| `single(fieldName)`            | One file under `fieldName`           | `ctx.file`                          |
| `array(fieldName, maxCount?)`  | Several files under `fieldName`      | `ctx.files` (array)                 |
| `fields([{ name, maxCount }])` | Files under several field names      | `ctx.files` (object keyed by field) |
| `any()`                        | Any files                            | `ctx.files` (array)                 |
| `none()`                       | Text fields only; a file is an error | `ctx.request.body`                  |

`multer.diskStorage` and `multer.memoryStorage` are re-exported. The middleware assigns `ctx.file` and `ctx.files` at run time without augmenting Koa's TypeScript types, so declare them on your context type if you use strict typing.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_koa-multipart-form.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/koa-multipart-form)
- [multer options](https://github.com/expressjs/multer#multeropts)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
