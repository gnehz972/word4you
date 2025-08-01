import fs from "fs";
import { WordExplanation } from "../types";
import { executeWordCliWithStatusUpdate, executeWordCli } from "./cliManager";

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
    let timestamp = "";

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
      } else if (line.match(/<!--.*-->/)) {
        const metadata = line.replace("<!--", "").replace("-->", "").trim().split(" ");

        for (let j = 0; j < metadata.length; j++) {
          const split = metadata[j].split("=");
          if (split[0] === "timestamp") {
            timestamp = split[1];
          }
        }
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
      timestamp: timestamp,
    };
  } catch (error) {
    console.error("Error parsing raw word explanation:", error);
    return null;
  }
}

export async function getWordExplanation(word: string): Promise<WordExplanation | null> {
  try {
    const output = await executeWordCli(["query", word, "--raw"]);
    return parseRawWordExplanation(output, word);
  } catch (error) {
    console.error("Error getting word explanation:", error);
    return null;
  }
}

export async function saveWordToVocabulary(
  content: string,
  onStatusUpdate?: (message: string) => void,
): Promise<boolean> {
  try {
    const result = await executeWordCliWithStatusUpdate(["save", content], { onStatusUpdate });

    if (!result) {
      console.error(`Save failed for content`, content);
    }

    return result;
  } catch (error) {
    console.error("Error in saveWordToVocabulary:", error);
    return false;
  }
}

export async function deleteWordFromVocabulary(
  timestamp: string,
  onStatusUpdate?: (message: string) => void,
): Promise<boolean> {
  try {
    const result = await executeWordCliWithStatusUpdate(["delete", timestamp], { onStatusUpdate });

    if (!result) {
      console.error(`Delete failed for timestamp: ${timestamp}`);
    }

    return result;
  } catch (error) {
    console.error("Error in deleteWordFromVocabulary:", error);
    return false;
  }
}

export async function updateWordInVocabulary(
  timestamp: string,
  content: string,
  onStatusUpdate?: (message: string) => void,
): Promise<boolean> {
  try {
    const result = await executeWordCliWithStatusUpdate(["update", timestamp, "--content", content], {
      onStatusUpdate,
    });

    if (!result) {
      console.error(`Update failed for timestamp: ${timestamp}`);
    }

    return result;
  } catch (error) {
    console.error("Error in updateWordInVocabulary:", error);
    return false;
  }
}

// Parse saved words from the vocabulary notebook
export function parseSavedWords(vocabularyPath: string): WordExplanation[] {
  try {
    if (!fs.existsSync(vocabularyPath)) {
      return [];
    }

    const content = fs.readFileSync(vocabularyPath, "utf8");
    const words: WordExplanation[] = [];

    // Split content by word sections (## word)
    const sections = content.split("\n---\n");

    for (const section of sections) {
      if (!section.trim()) continue;

      const lines = section.split("\n");
      const wordLine = lines[0];
      const wordMatch = wordLine.match(/^## (.+)$/);

      if (!wordMatch) continue;

      const word = wordMatch[1].trim();
      const wordContent = lines.slice(1).join("\n");

      // Parse the word content similar to the original parser
      const parsed = parseRawWordExplanation(wordContent, word);
      if (parsed) {
        words.push({
          ...parsed,
        });
      }
    }

    return words;
  } catch (error) {
    console.error("Error parsing saved words:", error);
    return [];
  }
}
