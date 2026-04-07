import typescript from "@rollup/plugin-typescript";
import terser from "@rollup/plugin-terser";

export default {
  input: "src/core.ts",
  output: {
    file: "dist/loloverlay.js",
    format: "iife",
    name: "LOLOverlay",
    sourcemap: true,
    exports: "auto",
  },
  plugins: [
    typescript({
      tsconfig: "./tsconfig.json",
      declaration: false,
    }),
    terser(),
  ],
};
