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
  /** Vocabulary Base Directory - Base directory where 'word4you' subdirectory will be created (leave empty for default: home directory) */
  "vocabularyBaseDir": string,
  /** Enable Git Operations - Enable automatic Git commit and push operations when saving words */
  "gitEnabled": boolean,
  /** Git Remote URL - Git repository URL for vocabulary notebook backup (SSH URLs only) */
  "gitRemoteUrl"?: string,
  /** SSH Private Key Path - Path to SSH private key file for Git authentication (leave empty for default: ~/.ssh/id_ed25519) */
  "sshPrivateKeyPath": string,
  /** SSH Public Key Path - Path to SSH public key file for Git authentication (leave empty for default: ~/.ssh/id_ed25519.pub) */
  "sshPublicKeyPath": string,
  /** Gemini Model Name - Gemini model name to use for AI explanations (leave empty for default: gemini-2.0-flash-001) */
  "geminiModelName": string
}
}

declare namespace Arguments {
  /** Arguments passed to the `learn-word` command */
  export type LearnWord = {
  /** word */
  "word": string
}
}

