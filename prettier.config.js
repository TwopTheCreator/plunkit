/** @type {import("prettier").Config} */
module.exports = {
  // Use semicolons at the end of statements
  semi: true,

  // Use single quotes instead of double quotes
  singleQuote: true,

  // Include trailing commas wherever possible (ES5+)
  trailingComma: 'all',

  // Maximum line width
  printWidth: 100,

  // Number of spaces per indentation level
  tabWidth: 2,

  // Use spaces instead of tabs
  useTabs: false,

  // Format embedded HTML, CSS, and JS in Markdown
  embeddedLanguageFormatting: 'auto',

  // End files with a newline
  endOfLine: 'lf',

  // Quote props in objects only when required
  quoteProps: 'as-needed',
};
