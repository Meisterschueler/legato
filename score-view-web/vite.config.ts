import type { Plugin } from "vite";
import { defineConfig } from "vitest/config";

// WebViews load the bundle from file://, where module scripts cannot be fetched:
// put the JavaScript directly into index.html.
function inlineScripts(): Plugin {
  return {
    name: "legato-inline-scripts",
    apply: "build",
    enforce: "post",
    // After vite:license, which reads the modules of the chunks removed here.
    generateBundle: {
      order: "post",
      handler(_options, bundle) {
        const html = bundle["index.html"];
        if (html?.type !== "asset" || typeof html.source !== "string") return;
        let source = html.source;
        for (const [fileName, chunk] of Object.entries(bundle)) {
          if (chunk.type !== "chunk") continue;
          const tag = new RegExp(
            `<script type="module" crossorigin src="\\./${fileName}"></script>`,
          );
          const code = chunk.code.replaceAll("</script", "<\\/script");
          source = source.replace(
            tag,
            () => `<script type="module">${code}</script>`,
          );
          delete bundle[fileName];
        }
        html.source = source;
      },
    },
  };
}

export default defineConfig({
  base: "./",
  plugins: [inlineScripts()],
  build: {
    outDir: "../app/assets/score-view-web",
    emptyOutDir: true,
    modulePreload: false,
    // Shown in the app's license page (T098).
    license: { fileName: "licenses.json" },
  },
  test: {
    passWithNoTests: true,
  },
});
