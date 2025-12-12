import path from "node:path";
import { fileURLToPath } from "node:url";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";
import replace from "@rollup/plugin-replace";
import typescript from "@rollup/plugin-typescript";
import copy from "rollup-plugin-copy";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default {
  input: path.join(__dirname, "src/index.ts"),
  output: {
    file: path.join(__dirname, "dist/bundle.js"),
    format: "esm",
    sourcemap: true,
  },
  plugins: [
    resolve({ browser: true, preferBuiltins: false }),
    commonjs(),
    json(),
    copy({
      targets: [
        {
          src: path.join(__dirname, "../../crates/html-to-markdown-wasm/dist-web/html_to_markdown_wasm_bg.wasm"),
          dest: path.join(__dirname, "dist"),
        },
      ],
    }),
    typescript({ tsconfig: path.join(__dirname, "tsconfig.json"), sourceMap: true }),
    replace({
      preventAssignment: true,
      "process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV ?? "development"),
    }),
  ],
  onwarn(warning, warn) {
    if (warning.code === "CIRCULAR_DEPENDENCY" && /html_to_markdown_wasm_bg/.test(warning.message)) {
      return;
    }
    warn(warning);
  },
};
