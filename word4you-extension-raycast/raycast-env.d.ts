/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `learn-word` command */
  export type LearnWord = ExtensionPreferences & {
  /** Gemini API Key - Your Google Gemini API key (get one at https://aistudio.google.com/app/apikey) */
  "geminiApiKey": string,
  /** Gemini Model Name - The Gemini model to use for word explanations */
  "geminiModelName": string,
  /** Vocabulary Base Directory - Base directory where vocabulary files will be stored (~ for home directory) */
  "vocabularyBaseDir": string,
  /** Enable Git Integration - Enable Git version control for your vocabulary files */
  "gitEnabled": boolean,
  /** Git Remote URL - Git remote repository URL (only used if Git integration is enabled) */
  "gitRemoteUrl": string
}
}

declare namespace Arguments {
  /** Arguments passed to the `learn-word` command */
  export type LearnWord = {
  /** word */
  "word": string
}
}

