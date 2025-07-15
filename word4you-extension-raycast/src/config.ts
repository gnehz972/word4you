import { getPreferenceValues } from "@raycast/api";
import path from "path";
import os from "os";

export interface Preferences {
  geminiApiKey: string;
  vocabularyBaseDir: string;
  gitRemoteUrl: string;
  sshPrivateKeyPath: string;
  sshPublicKeyPath: string;
}

// Cross-platform path resolution for vocabulary file
export function getVocabularyPath(baseDir?: string): string {
  const vocabularyBaseDir = baseDir || os.homedir();
  return path.join(vocabularyBaseDir, "word4you", "vocabulary_notebook.md");
}

// Ensure directory exists for vocabulary file
export function ensureVocabularyDirectoryExists(vocabularyPath: string): void {
  try {
    const dir = path.dirname(vocabularyPath);
    if (!require("fs").existsSync(dir)) {
      require("fs").mkdirSync(dir, { recursive: true });
    }
  } catch (error) {
    console.error("Error creating vocabulary directory:", error);
  }
}

// Get executable path for the word4you CLI
export function getExecutablePath(): string {
  return path.join(__dirname, "assets/word4you");
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
    GEMINI_API_KEY: preferences.geminiApiKey,
    VOCABULARY_BASE_DIR: preferences.vocabularyBaseDir || os.homedir(),
    ...(preferences.gitRemoteUrl && {
      GIT_REMOTE_URL: preferences.gitRemoteUrl,
    }),
    ...(preferences.sshPrivateKeyPath && {
      SSH_PRIVATE_KEY_PATH: preferences.sshPrivateKeyPath,
    }),
    ...(preferences.sshPublicKeyPath && {
      SSH_PUBLIC_KEY_PATH: preferences.sshPublicKeyPath,
    }),
  };
}
