// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: true },
  modules: ["@nuxt/ui"],
  ssr: false,
  // https://github.com/nuxt/ui/issues/6118
  hooks: {
    "imports:extend"(imports) {
      for (let index = imports.length - 1; index >= 0; index -= 1) {
        const item = imports[index];
        if (!item) {
          continue;
        }
        if (
          item.name === "options" &&
          typeof item.from === "string" &&
          item.from.includes("/useResizable")
        ) {
          imports.splice(index, 1);
        }
      }
    },
  },
  ui: {
    fonts: false,
  },
});
