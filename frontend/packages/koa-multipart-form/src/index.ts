/* eslint @typescript-eslint/no-explicit-any: 0 */

// Code Pulled from https://github.com/koajs/multer
// Fixes issues with esm.  This is a temporary fix until the original package is updated to support esm properly and not conflict tsx.

import type Koa from "koa";
import originalMulter from "multer";

interface Field {
  /** The field name. */
  name: string;
  /** Optional maximum number of files per field to accept. */
  maxCount?: number;
}

interface Instance {
  /** In case you need to handle a text-only multipart form, you can use any of the multer methods (.single(), .array(), fields()), req.body contains the text fields */
  /** Accept a single file with the name fieldName. The single file will be stored in req.file. */
  single(fieldName?: string): Koa.Middleware;
  /** Accept an array of files, all with the name fieldName. Optionally error out if more than maxCount files are uploaded. The array of files will be stored in req.files. */
  array(fieldName: string, maxCount?: number): Koa.Middleware;
  /** Accept a mix of files, specified by fields. An object with arrays of files will be stored in req.files. */
  fields(fields: Field[]): Koa.Middleware;
  /** Accepts all files that comes over the wire. An array of files will be stored in req.files. */
  any(): Koa.Middleware;
  /** No files accepted. */
  none(): Koa.Middleware;
}

export function multer(options?: originalMulter.Options): Instance {
  const m = originalMulter(options);

  makePromise(m, "any");
  makePromise(m, "array");
  makePromise(m, "fields");
  makePromise(m, "none");
  makePromise(m, "single");

  return m as originalMulter.Multer & Instance;
}

function makePromise(
  multer: originalMulter.Multer & Record<string, any>,
  name: string,
) {
  if (!multer[name]) return;

  const fn = multer[name];

  multer[name] = function () {
    // eslint-disable-next-line prefer-rest-params
    const middleware = Reflect.apply(fn, this, arguments);

    return async (ctx: Koa.Context, next: () => any) => {
      if (ctx.header["content-type"] !== "multipart/form-data") return next();
      await new Promise((resolve, reject) => {
        // Ignore if not multipart/form-data
        middleware(ctx.req, ctx.res, (err: any) => {
          if (err) return reject(err);
          if ("request" in ctx) {
            // @ts-ignore
            if (ctx.req.body) {
              // @ts-ignore
              ctx.request.body = ctx.req.body;
              // @ts-ignore
              delete ctx.req.body;
            }

            // @ts-ignore
            if (ctx.req.file) {
              // @ts-ignore
              ctx.request.file = ctx.req.file;
              // @ts-ignore
              ctx.file = ctx.req.file;
              // @ts-ignore
              delete ctx.req.file;
            }

            // @ts-ignore
            if (ctx.req.files) {
              // @ts-ignore
              ctx.request.files = ctx.req.files;
              // @ts-ignore
              ctx.files = ctx.req.files;
              // @ts-ignore
              delete ctx.req.files;
            }
          }

          resolve(ctx);
        });
      });

      return next();
    };
  };
}

multer.diskStorage = originalMulter.diskStorage;
multer.memoryStorage = originalMulter.memoryStorage;
