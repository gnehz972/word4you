import { getPreferenceValues, environment } from "@raycast/api";
import path from "path";
import os from "os";
import fs from "fs";

export interface Preferences {
  geminiApiKey: string;
  geminiModelName: string;
  vocabularyBaseDir: string;
  gitEnabled: boolean;
  gitRemoteUrl: string;
}

// CLI Download Configuration
export const CLI_CONFIG = {
  // GitHub release version and base URL
  version: "v1.0.0",
  baseUrl: "https://github.com/gnehz972/word4you/releases/download",

  // Platform-specific asset names
  assets: {
    "darwin-arm64": "word4you-aarch64-apple-darwin",
    "darwin-x64": "word4you-x86_64-apple-darwin",
    "linux-x64": "word4you-x86_64-unknown-linux-gnu",
  },

  // Expected SHA256 hashes for verification
  hashes: {
    "word4you-aarch64-apple-darwin": "ca92435862631cc4a98239f00c3ca596dbd6231239a47e1d2fbb28a6fec36509",
    "word4you-x86_64-apple-darwin": "f51ab9cc5e84c2648b1e6976c544e25c5cb26cc480bd94144fa62ae0a9ae7a85",
    "word4you-x86_64-unknown-linux-gnu": "ad3d81dbbba374157816b63005b129abc35735dc813159ebf6e76a50fbcdf425",
  },
} as const;

// Get the path to the Word4You CLI executable
export function getCliFilepath(): string {
  const dir = path.join(environment.supportPath, "cli");
  return path.join(dir, "word4you");
}

// Get download URL for current platform
export function getDownloadUrl(): { url: string; assetName: string; expectedHash: string } {
  const platform = os.platform();
  const arch = os.arch();

  // Only support macOS and Linux
  if (platform !== "darwin" && platform !== "linux") {
    throw new Error(`Unsupported platform: ${platform}. Only macOS and Linux are supported.`);
  }

  // Map platform and architecture to asset name
  let platformKey: keyof typeof CLI_CONFIG.assets;

  if (platform === "darwin") {
    platformKey = arch === "arm64" ? "darwin-arm64" : "darwin-x64";
  } else if (platform === "linux" && arch === "x64") {
    platformKey = "linux-x64";
  } else {
    throw new Error(`Unsupported architecture: ${arch} on Linux. Only x64 and arm64 are supported.`);
  }

  const assetName = CLI_CONFIG.assets[platformKey];
  const url = `${CLI_CONFIG.baseUrl}/${CLI_CONFIG.version}/${assetName}`;
  const expectedHash = CLI_CONFIG.hashes[assetName];

  return { url, assetName, expectedHash };
}

// Get the default vocabulary path from the CLI's configuration
export function getVocabularyPath(): string {
  // Use a fixed path to avoid issues with CLI path resolution
  return path.join(os.homedir(), "word4you", "vocabulary_notebook.md");
}

// Ensure directory exists for vocabulary file
export function ensureVocabularyDirectoryExists(vocabularyPath: string): void {
  try {
    const dir = path.dirname(vocabularyPath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  } catch (err) {
    console.error("Error creating vocabulary directory:", err);
  }
}

// Get preferences with proper typing
export function getPreferences(): Preferences {
  return getPreferenceValues<Preferences>();
}

// Create environment variables from preferences
export function createEnvironmentFromPreferences(): NodeJS.ProcessEnv {
  const preferences = getPreferences();

  return {
    ...process.env,
    // Pass Raycast preferences as environment variables for the CLI
    WORD4YOU_GEMINI_API_KEY: preferences.geminiApiKey || "",
    WORD4YOU_GEMINI_MODEL_NAME: preferences.geminiModelName || "gemini-2.0-flash-001",
    WORD4YOU_VOCABULARY_BASE_DIR: preferences.vocabularyBaseDir || "~",
    WORD4YOU_GIT_ENABLED: preferences.gitEnabled ? "true" : "false",
    WORD4YOU_GIT_REMOTE_URL: preferences.gitRemoteUrl || "",
  };
}
