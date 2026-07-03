import type { StorybookConfig } from "@storybook/react-webpack5";

import { dirname } from "path";

import { fileURLToPath } from "url";

/**
 * This function is used to resolve the absolute path of a package.
 * It is needed in projects that use Yarn PnP or are set up within a monorepo.
 */
function getAbsolutePath(value: string): any {
  return dirname(fileURLToPath(import.meta.resolve(`${value}/package.json`)));
}

const config: StorybookConfig = {
  stories: ["../src/**/*.mdx", "../src/**/*.stories.@(js|jsx|mjs|ts|tsx)"],
  staticDirs: ["../public"],
  addons: [
    getAbsolutePath("@storybook/addon-links"),
    getAbsolutePath("@storybook/addon-onboarding"),
    {
      name: getAbsolutePath("@storybook/addon-styling-webpack"),
      options: {
        // Check out https://github.com/storybookjs/addon-styling-webpack#readme
        // for more details on this addon's options.
        rules: [
          {
            test: /\.css$/,
            use: [
              "style-loader",
              {
                loader: "css-loader",
                options: { importLoaders: 1 },
              },
              {
                // Picks up postcss.config.js in the project root (tailwindcss, autoprefixer).
                loader: "postcss-loader",
                options: { implementation: getAbsolutePath("postcss") },
              },
            ],
          },
        ],
      },
    },
    getAbsolutePath("@storybook/addon-webpack5-compiler-babel"),
    getAbsolutePath("@chromatic-com/storybook"),
    getAbsolutePath("@storybook/addon-docs"),
  ],

  framework: {
    name: getAbsolutePath("@storybook/react-webpack5"),
    options: {},
  },

  async babel(config, { configType }) {
    return {
      sourceType: "unambiguous",
      presets: [
        [
          "@babel/preset-env",
          {
            targets: {
              chrome: 100,
              safari: 15,
              firefox: 91,
            },
          },
        ],
        "@babel/preset-typescript",
        "@babel/preset-react",
      ],
      plugins: ["@babel/plugin-syntax-import-attributes"],
    };
  },

  docs: {},

  typescript: {
    reactDocgen: "react-docgen-typescript",
  },
};
export default config;
