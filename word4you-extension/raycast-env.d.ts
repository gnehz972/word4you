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
  /** Gemini API Key - Your Google Gemini API key for AI word explanations */
  "geminiApiKey": string,
  /** Vocabulary Notebook File - Path to your vocabulary notebook markdown file */
  "vocabularyFile": string,
  /** Git Remote URL - Git repository URL for vocabulary notebook backup (optional) */
  "gitRemoteUrl"?: string
}
}

declare namespace Arguments {
  /** Arguments passed to the `learn-word` command */
  export type LearnWord = {}
}

