import { create } from "storybook/theming";

export default create({
  base: "light",
  brandTitle: "HasteHealth Components",
  brandUrl: "https://haste.health",
  brandImage: "https://haste.health/img/logo_text.svg",
  brandTarget: "_self",

  colorPrimary: "#0f766e",
  colorSecondary: "#115e59",

  // UI
  appBg: "#f5fcfb",
  appContentBg: "#ffffff",
  appPreviewBg: "#ffffff",
  appBorderColor: "#0f766e",
  appBorderRadius: 4,

  // Text colors
  textColor: "#134e4a",
  textInverseColor: "#ffffff",

  // Toolbar default and active colors
  barTextColor: "#0f766e",
  barSelectedColor: "#115e59",
  barHoverColor: "#134e4a",
  barBg: "#ffffff",

  // Form colors
  inputBg: "#ffffff",
  inputBorder: "#0f766e",
  inputTextColor: "#134e4a",
  inputBorderRadius: 2,
});
