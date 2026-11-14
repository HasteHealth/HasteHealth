import typescript from "@rollup/plugin-typescript";
import { dts } from "rollup-plugin-dts";

const external = (id) => id != "tslib" && !/^(\.?\/|\w:)/.test(id);

export default [
  {
    input: "src/index.ts",
    external,
    output: [
      { file: "lib/index.cjs", format: "cjs", exports: "named" },
      { file: "lib/index.js", format: "es" },
    ],
    plugins: [typescript({ declaration: false, sourceMap: false })],
  },
  {
    input: "src/index.ts",
    external,
    output: [
      { file: "lib/index.d.ts", format: "es" },
      { file: "lib/index.d.cts", format: "es" },
    ],
    plugins: [dts()],
  },
];
