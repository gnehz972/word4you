import { getPreferenceValues } from "@raycast/api";
import path from "path";
import os from "os";
import { execSync } from "child_process";
import fs from "fs";

export interface Preferences {
  cliPath: string;
}

// Get the default vocabulary path from the CLI's configuration
export function getVocabularyPath(): string {
  try {
    // Try to get the path from the CLI's configuration using the --show-vob-path flag
    const executablePath = getExecutablePath();
    const output = execSync(`${executablePath} config --show-vob-path`, {
      encoding: "utf8",
    }).trim();

    // If we got a valid path, return it
    if (output && output.length > 0) {
      return output;
    }

    // Fallback to default path if the command didn't return a valid path
    return path.join(os.homedir(), "word4you", "vocabulary_notebook.md");
  } catch (error) {
    console.error("Error getting vocabulary path from CLI:", error);
    // Fallback to default path
    return path.join(os.homedir(), "word4you", "vocabulary_notebook.md");
  }
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

// Check if word4you CLI is installed
export function isCliInstalled(): boolean {
  try {
    const preferences = getPreferences();

    // If user specified a custom path, check if it exists
    if (preferences.cliPath) {
      return fs.existsSync(preferences.cliPath);
    }

    // Otherwise check if it's in PATH
    execSync("which word4you", { stdio: "ignore" });
    return true;
  } catch (_) {
    // We don't need the error object, just return false if any error occurs
    return false;
  }
}

// Get executable path for the word4you CLI
export function getExecutablePath(): string {
  const preferences = getPreferences();

  // If user specified a custom path, use it
  if (preferences.cliPath) {
    return preferences.cliPath;
  }

  // Otherwise assume it's in PATH
  return "word4you";
}

// Get preferences with proper typing
export function getPreferences(): Preferences {
  return getPreferenceValues<Preferences>();
}

// Create environment variables from preferences
export function createEnvironmentFromPreferences(): NodeJS.ProcessEnv {
  // Use the CLI's own configuration
  return process.env;
}
