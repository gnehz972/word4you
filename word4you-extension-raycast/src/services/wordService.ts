import { execSync, spawn } from "child_process";
import fs from "fs";
import path from "path";
import { WordExplanation, SavedWord } from "../types";
import {
  getPreferences,
  getVocabularyPath,
  getExecutablePath,
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
    const executablePath = getExecutablePath();

    // Use cross-platform path resolution for vocabulary file
    const vocabularyPath = getVocabularyPath();

    // Ensure the directory exists
    ensureVocabularyDirectoryExists(vocabularyPath);

    // Create environment variables from preferences
    const env = createEnvironmentFromPreferences();

    // Use --raw flag to get clean output without TTY interaction
    const command = `"${executablePath}" query "${word}" --raw`;

    // If using a custom path, use the directory of the executable as cwd
    // Otherwise, use the current directory
    const cwd = preferences.cliPath ? path.dirname(preferences.cliPath) : process.cwd();

    const output = execSync(command, {
      encoding: "utf8",
      timeout: 30000,
      cwd: cwd,
      env: env,
    });

    return parseRawWordExplanation(output, word);
  } catch (error) {
    console.error("Error getting word explanation:", error);
    return null;
  }
}

export async function saveWordToVocabulary(
  word: string,
  content: string,
  onStatusUpdate?: (message: string) => void,
): Promise<boolean> {
  return new Promise((resolve) => {
    try {
      const preferences = getPreferences();
      const executablePath = getExecutablePath();

      // Use cross-platform path resolution for vocabulary file
      const vocabularyPath = getVocabularyPath();

      // Ensure the directory exists
      ensureVocabularyDirectoryExists(vocabularyPath);

      // Create environment variables from preferences
      const env = createEnvironmentFromPreferences();

      // If using a custom path, use the directory of the executable as cwd
      // Otherwise, use the current directory
      const cwd = preferences.cliPath ? path.dirname(preferences.cliPath) : process.cwd();

      // Use spawn to capture real-time output
      const child = spawn(executablePath, ["save", word, "--content", content], {
        cwd: cwd,
        env: env,
        stdio: ["pipe", "pipe", "pipe"],
      });

      let fullOutput = "";
      let errorOutput = "";
      let success = false;

      child.stdout.on("data", (data) => {
        const message = data.toString();
        fullOutput += message;
        if (onStatusUpdate) {
          onStatusUpdate(message.trim());
        }
      });

      child.stderr.on("data", (data) => {
        const message = data.toString();
        errorOutput += message;
        fullOutput += message;
        if (onStatusUpdate) {
          onStatusUpdate(message.trim());
        }
      });

      child.on("close", (code) => {
        console.log(`Process closed with code: ${code}`);
        console.log(`Full output: ${fullOutput}`);
        console.log(`Error output: ${errorOutput}`);

        if (code !== 0) {
          console.error(`Save failed for word: ${word}`);
          console.error(`content: ${content}`);
          console.error(`Error details: ${errorOutput}`);
        }

        success = code === 0;
        resolve(success);
      });

      child.on("error", (error) => {
        console.error("Error spawning process:", error);
        resolve(false);
      });
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
  return new Promise((resolve) => {
    try {
      const preferences = getPreferences();
      const executablePath = getExecutablePath();

      // Use cross-platform path resolution for vocabulary file
      const vocabularyPath = getVocabularyPath();

      // Ensure the directory exists
      ensureVocabularyDirectoryExists(vocabularyPath);

      // Create environment variables from preferences
      const env = createEnvironmentFromPreferences();

      // Prepare delete command arguments
      const args = timestamp ? ["delete", word, "--timestamp", timestamp] : ["delete", word];

      // If using a custom path, use the directory of the executable as cwd
      // Otherwise, use the current directory
      const cwd = preferences.cliPath ? path.dirname(preferences.cliPath) : process.cwd();

      // Use spawn to capture real-time output
      const child = spawn(executablePath, args, {
        cwd: cwd,
        env: env,
        stdio: ["pipe", "pipe", "pipe"],
      });

      let fullOutput = "";
      let success = false;

      child.stdout.on("data", (data) => {
        const message = data.toString();
        fullOutput += message;
        if (onStatusUpdate) {
          onStatusUpdate(message.trim());
        }
      });

      child.stderr.on("data", (data) => {
        const message = data.toString();
        fullOutput += message;
        if (onStatusUpdate) {
          onStatusUpdate(message.trim());
        }
      });

      child.on("close", (code) => {
        success = code === 0;
        resolve(success);
      });

      child.on("error", (error) => {
        console.error("Error spawning process:", error);
        resolve(false);
      });
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
  return new Promise((resolve) => {
    try {
      const preferences = getPreferences();
      const executablePath = getExecutablePath();

      // Use cross-platform path resolution for vocabulary file
      const vocabularyPath = getVocabularyPath();

      // Ensure the directory exists
      ensureVocabularyDirectoryExists(vocabularyPath);

      // Create environment variables from preferences
      const env = createEnvironmentFromPreferences();

      // Prepare update command arguments
      const args = timestamp
        ? ["update", word, "--content", content, "--timestamp", timestamp]
        : ["update", word, "--content", content];

      // If using a custom path, use the directory of the executable as cwd
      // Otherwise, use the current directory
      const cwd = preferences.cliPath ? path.dirname(preferences.cliPath) : process.cwd();

      // Use spawn to capture real-time output
      const child = spawn(executablePath, args, {
        cwd: cwd,
        env: env,
        stdio: ["pipe", "pipe", "pipe"],
      });

      let fullOutput = "";
      let success = false;

      child.stdout.on("data", (data) => {
        const message = data.toString();
        fullOutput += message;
        if (onStatusUpdate) {
          onStatusUpdate(message.trim());
        }
      });

      child.stderr.on("data", (data) => {
        const message = data.toString();
        fullOutput += message;
        if (onStatusUpdate) {
          onStatusUpdate(message.trim());
        }
      });

      child.on("close", (code) => {
        success = code === 0;
        resolve(success);
      });

      child.on("error", (error) => {
        console.error("Error spawning process:", error);
        resolve(false);
      });
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
