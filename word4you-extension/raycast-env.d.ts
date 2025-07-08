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
  /** Word4You Executable Path - Path to the word4you executable (optional, defaults to extension directory) */
  "executablePath"?: string
}
}

declare namespace Arguments {
  /** Arguments passed to the `learn-word` command */
  export type LearnWord = {}
}

