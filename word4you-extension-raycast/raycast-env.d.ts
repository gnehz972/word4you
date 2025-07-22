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
  /** Word4You CLI Path - Path to the Word4You CLI executable (leave empty if installed in PATH) */
  "cliPath": string
}
}

declare namespace Arguments {
  /** Arguments passed to the `learn-word` command */
  export type LearnWord = {
  /** word */
  "word": string
}
}

