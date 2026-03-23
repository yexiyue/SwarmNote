import { defineConfig } from "@lingui/cli";

export default defineConfig({
  sourceLocale: "zh",
  locales: ["zh", "en"],
  catalogs: [
    {
      path: "<rootDir>/src/locales/{locale}/messages",
      include: ["src"],
    },
  ],
});
