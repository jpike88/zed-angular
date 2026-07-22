# Zed Angular Extension

## Overview

**Note: This project is currently a work in progress. Expect potential bugs or issues.**

This extension integrates the Angular Language Service into Zed. It uses the same options that Angular applies during compilation. To ensure the most accurate information, enable the `strictTemplates` option in the `tsconfig.json` of the angular project as shown in below:

```json
"angularCompilerOptions": {
  "strictTemplates": true
}
```

## Version Management

The extension depends on `@angular/language-service` and `typescript` Node packages. It will use whatever versions of each that are available locally in your project.

Please ensure your project's Angular and TypeScript versions are compatible to avoid issues.

Refer to [Angular Version Compatibility](https://angular.dev/reference/versions#unsupported-angular-versions) for details.

## Installation Instructions

To install this extension locally:

1. Clone this repository.
2. Open the Zed editor and navigate to the Extensions window.
3. Click on "Install Dev Extension."
4. Select the cloned repository location and complete the installation.
5. Add a language server list definition to the HTML and TypeScript language settings. In `settings.json`, add the following _(ellipsis is a valid value in settings, use it as shown)_:

```json
{
  "languages": {
    "TypeScript": {
      "language_servers": ["angular", "..."]
    },
    "HTML": {
      "language_servers": ["angular", "..."]
    }
  }
}
```
