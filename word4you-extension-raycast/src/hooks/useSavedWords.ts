import { useState, useEffect } from "react";
import { Toast, showToast } from "@raycast/api";
import { SavedWord } from "../types";
import { parseSavedWords } from "../services/wordService";
import { getVocabularyPath } from "../config";

export function useSavedWords() {
  const [savedWords, setSavedWords] = useState<SavedWord[]>([]);
  const [isLoadingSaved, setIsLoadingSaved] = useState(true);
  const [savedWordsMap, setSavedWordsMap] = useState<Map<string, SavedWord>>(new Map());

  const loadSavedWords = async () => {
    try {
      const vocabularyPath = getVocabularyPath();
      const words = parseSavedWords(vocabularyPath);
      setSavedWords(words);

      // Create a map for quick lookup
      const wordsMap = new Map<string, SavedWord>();
      words.forEach((word) => wordsMap.set(word.word.toLowerCase(), word));
      setSavedWordsMap(wordsMap);
    } catch (error) {
      console.error("Error loading saved words:", error);
      await showToast({
        style: Toast.Style.Failure,
        title: "Error",
        message: "Failed to load saved words",
      });
    } finally {
      setIsLoadingSaved(false);
    }
  };

  // Load saved words when CLI is installed
  useEffect(() => {
    loadSavedWords();
  }, []);

  return {
    savedWords,
    isLoadingSaved,
    savedWordsMap,
    loadSavedWords,
  };
}
