import { execSync } from "child_process";
import fs from "fs";
import path from "path";
import { WordExplanation, SavedWord } from "../types";
import {
  getPreferences,
  getVocabularyPath,
  getExecutablePathAsync,
  createEnvironmentFromPreferences,
  ensureVocabularyDirectoryExists,
} from "../config";

export function parseRawWordExplanation(output: string, word: string): WordExplanation | null {
  try {
    const lines = output
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line);

    let pronunciation = "";
    let definition = "";
    let chinese = "";
    let example_en = "";
    let example_zh = "";
    let tip = "";

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];

      // Pronunciation: */pronunciation/*
      if (line.match(/^\*\/.*\/\*$/)) {
        pronunciation = line.replace(/^\*\//, "").replace(/\/\*$/, "");
      }

      // Definition: > Definition text
      else if (line.startsWith("> ")) {
        definition = line.replace(/^> /, "");
      }

      // Chinese: **Chinese text**
      else if (line.match(/^\*\*.*\*\*$/)) {
        chinese = line.replace(/^\*\*/, "").replace(/\*\*$/, "");
      }

      // Examples: - Example text
      else if (line.startsWith("- ")) {
        const exampleText = line.replace(/^- /, "");
        if (!/[\u4e00-\u9fa5]/.test(exampleText) && !example_en) {
          example_en = exampleText;
        } else if (/[\u4e00-\u9fa5]/.test(exampleText) && !example_zh) {
          example_zh = exampleText;
        }
      }

      // Tip: *Tip text* (but not pronunciation format)
      else if (line.match(/^\*.*\*$/) && !line.match(/^\*\/.*\/\*$/)) {
        tip = line.replace(/^\*/, "").replace(/\*$/, "");
      }
    }

    return {
      word: word,
      pronunciation: pronunciation || "",
      definition: definition || "",
      chinese: chinese || "",
      example_en: example_en || "",
      example_zh: example_zh || "",
      tip: tip || "",
      raw_output: output,
    };
  } catch (error) {
    console.error("Error parsing raw word explanation:", error);
    return null;
  }
}

export async function getWordExplanation(word: string): Promise<WordExplanation | null> {
  try {
    const preferences = getPreferences();

    // Get executable path, downloading if necessary
    const executablePath = await getExecutablePathAsync();

    // Use cross-platform path resolution for vocabulary file
    const vocabularyPath = getVocabularyPath();

    // Ensure the directory exists
    ensureVocabularyDirectoryExists(vocabularyPath);

    // Create environment variables from preferences
    const env = createEnvironmentFromPreferences();

    // If using a custom path, use the directory of the executable as cwd
    // Otherwise, use the current directory
    const cwd = preferences.cliPath ? path.dirname(preferences.cliPath) : process.cwd();

    // Properly escape the executable path and arguments for shell execution
    const escapedPath =
      executablePath.includes(" ") || executablePath.includes("(")
        ? `'${executablePath.replace(/'/g, "'\\''")}'`
        : executablePath;

    const command = `${escapedPath} query "${word.replace(/"/g, '\\"')}" --raw`;

    const output = execSync(command, {
      encoding: "utf8",
      timeout: 30000,
      cwd: cwd,
      env: env,
    });

    return parseRawWordExplanation(output, word);
  } catch (error: any) {
    console.error("Error getting word explanation:", error);
    return null;
  }
}

export async function saveWordToVocabulary(
  word: string,
  content: string,
  onStatusUpdate?: (message: string) => void,
): Promise<boolean> {
  return new Promise(async (resolve) => {
    try {
      const preferences = getPreferences();

      // Get executable path, downloading if necessary
      const executablePath = await getExecutablePathAsync();

      // Use cross-platform path resolution for vocabulary file
      const vocabularyPath = getVocabularyPath();

      // Ensure the directory exists
      ensureVocabularyDirectoryExists(vocabularyPath);

      // Create environment variables from preferences
      const env = createEnvironmentFromPreferences();

      // If using a custom path, use the directory of the executable as cwd
      // Otherwise, use the current directory
      const cwd = preferences.cliPath ? path.dirname(preferences.cliPath) : process.cwd();

      // Properly escape the executable path and arguments for shell execution
      const escapedPath =
        executablePath.includes(" ") || executablePath.includes("(")
          ? `'${executablePath.replace(/'/g, "'\\''")}'`
          : executablePath;

      const command = `${escapedPath} save "${word.replace(/"/g, '\\"')}" --content "${content.replace(/"/g, '\\"')}"`;

      try {
        const output = execSync(command, {
          encoding: "utf8",
          timeout: 30000,
          cwd: cwd,
          env: env,
        });

        if (onStatusUpdate) {
          onStatusUpdate(output.trim());
        }

        resolve(true);
      } catch (error: any) {
        console.error(`Save failed for word: ${word}`);
        console.error(`content: ${content}`);
        console.error(`Error details:`, error);

        if (onStatusUpdate) {
          onStatusUpdate(`Error: ${error.message}`);
        }

        resolve(false);
      }
    } catch (error) {
      console.error("Error in saveWordToVocabulary:", error);
      resolve(false);
    }
  });
}

export async function deleteWordFromVocabulary(
  word: string,
  timestamp?: string,
  onStatusUpdate?: (message: string) => void,
): Promise<boolean> {
  return new Promise(async (resolve) => {
    try {
      const preferences = getPreferences();

      // Get executable path, downloading if necessary
      const executablePath = await getExecutablePathAsync();

      // Use cross-platform path resolution for vocabulary file
      const vocabularyPath = getVocabularyPath();

      // Ensure the directory exists
      ensureVocabularyDirectoryExists(vocabularyPath);

      // Create environment variables from preferences
      const env = createEnvironmentFromPreferences();

      // If using a custom path, use the directory of the executable as cwd
      // Otherwise, use the current directory
      const cwd = preferences.cliPath ? path.dirname(preferences.cliPath) : process.cwd();

      // Properly escape the executable path and arguments for shell execution
      const escapedPath =
        executablePath.includes(" ") || executablePath.includes("(")
          ? `'${executablePath.replace(/'/g, "'\\''")}'`
          : executablePath;

      const timestampArg = timestamp ? ` --timestamp "${timestamp}"` : "";
      const command = `${escapedPath} delete "${word.replace(/"/g, '\\"')}"${timestampArg}`;

      try {
        const output = execSync(command, {
          encoding: "utf8",
          timeout: 30000,
          cwd: cwd,
          env: env,
        });

        if (onStatusUpdate) {
          onStatusUpdate(output.trim());
        }

        resolve(true);
      } catch (error: any) {
        console.error(`Delete failed for word: ${word}`);
        console.error(`Error details:`, error);

        if (onStatusUpdate) {
          onStatusUpdate(`Error: ${error.message}`);
        }

        resolve(false);
      }
    } catch (error) {
      console.error("Error in deleteWordFromVocabulary:", error);
      resolve(false);
    }
  });
}

export async function updateWordInVocabulary(
  word: string,
  content: string,
  timestamp?: string,
  onStatusUpdate?: (message: string) => void,
): Promise<boolean> {
  return new Promise(async (resolve) => {
    try {
      const preferences = getPreferences();

      // Get executable path, downloading if necessary
      const executablePath = await getExecutablePathAsync();

      // Use cross-platform path resolution for vocabulary file
      const vocabularyPath = getVocabularyPath();

      // Ensure the directory exists
      ensureVocabularyDirectoryExists(vocabularyPath);

      // Create environment variables from preferences
      const env = createEnvironmentFromPreferences();

      // If using a custom path, use the directory of the executable as cwd
      // Otherwise, use the current directory
      const cwd = preferences.cliPath ? path.dirname(preferences.cliPath) : process.cwd();

      // Properly escape the executable path and arguments for shell execution
      const escapedPath =
        executablePath.includes(" ") || executablePath.includes("(")
          ? `'${executablePath.replace(/'/g, "'\\''")}'`
          : executablePath;

      const timestampArg = timestamp ? ` --timestamp "${timestamp}"` : "";
      const command = `${escapedPath} update "${word.replace(/"/g, '\\"')}" --content "${content.replace(/"/g, '\\"')}"${timestampArg}`;

      try {
        const output = execSync(command, {
          encoding: "utf8",
          timeout: 30000,
          cwd: cwd,
          env: env,
        });

        if (onStatusUpdate) {
          onStatusUpdate(output.trim());
        }

        resolve(true);
      } catch (error: any) {
        console.error(`Update failed for word: ${word}`);
        console.error(`content: ${content}`);
        console.error(`Error details:`, error);

        if (onStatusUpdate) {
          onStatusUpdate(`Error: ${error.message}`);
        }

        resolve(false);
      }
    } catch (error) {
      console.error("Error in updateWordInVocabulary:", error);
      resolve(false);
    }
  });
}

// Parse saved words from the vocabulary notebook
export function parseSavedWords(vocabularyPath: string): SavedWord[] {
  try {
    if (!fs.existsSync(vocabularyPath)) {
      return [];
    }

    const content = fs.readFileSync(vocabularyPath, "utf8");
    const words: SavedWord[] = [];

    // Split content by word sections (## word)
    const sections = content.split(/(?=^## )/m);

    for (const section of sections) {
      if (!section.trim()) continue;

      const lines = section.split("\n");
      const wordLine = lines[0];
      const wordMatch = wordLine.match(/^## (.+)$/);

      if (!wordMatch) continue;

      const word = wordMatch[1].trim();
      const wordContent = lines.slice(1).join("\n");

      // Check for timestamp in the first two lines
      let timestamp = "";
      const timestampMatch = lines[1] && lines[1].match(/^Timestamp: (.+)$/);
      if (timestampMatch) {
        timestamp = timestampMatch[1];
      }

      // Parse the word content similar to the original parser
      const parsed = parseRawWordExplanation(wordContent, word);
      if (parsed) {
        words.push({
          ...parsed,
          timestamp: timestamp,
        });
      }
    }

    return words;
  } catch (error) {
    console.error("Error parsing saved words:", error);
    return [];
  }
}
