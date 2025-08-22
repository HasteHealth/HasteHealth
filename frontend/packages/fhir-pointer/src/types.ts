/* eslint-disable @typescript-eslint/no-explicit-any */
// Parent Loc
declare const __parent: unique symbol;
declare const __root: unique symbol;
type _Loc<B, T> = { [__root]: (root: B) => T };
export type Parent<B> = Loc<B, unknown, any> | null;

export type Loc<
  B,
  T,
  P extends Loc<B, unknown, Parent<B>> | null = null,
> = string & _Loc<B, T> & { [__parent]: P };

/*
 ** Access field in potentially Nullable V
 ** If V is nullable then set the return type to potentially nullable as well.
 */
export type NullGuard<V, Field extends keyof NonNullable<V>> =
  V extends NonNullable<V>
    ? NonNullable<V>[Field]
    : NonNullable<V>[Field] | undefined;
