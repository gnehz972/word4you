import { useState, useEffect } from "react";
import { Toast, showToast } from "@raycast/api";
import { WordExplanation, SavedWord } from "../types";
import {
  getWordExplanation,
  saveWordToVocabulary,
  deleteWordFromVocabulary,
  updateWordInVocabulary,
  parseSavedWords,
} from "../services/wordService";
import { getVocabularyPath } from "../config";
import { isCliInstalled, ensureCLI } from "../services/cliManager";

export function useWords(initialWord?: string) {
  const [searchText, setSearchText] = useState(initialWord || "");
  const [savedWords, setSavedWords] = useState<SavedWord[]>([]);
  const [aiResult, setAiResult] = useState<WordExplanation | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isLoadingSaved, setIsLoadingSaved] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [cliInstalled, setCliInstalled] = useState<boolean | null>(null);
  const [savedWordsMap, setSavedWordsMap] = useState<Map<string, SavedWord>>(new Map());

  // Check if CLI is installed, and download it if not
  useEffect(() => {
    const checkCliInstallation = async () => {
      const installed = isCliInstalled();

      if (!installed) {
        const toast = await showToast({
          style: Toast.Style.Animated,
          title: "Word4You CLI not found",
          message: "Downloading CLI...",
        });

        try {
          // Try to download the CLI
          await ensureCLI();
          toast.style = Toast.Style.Success;
          toast.title = "Word4You CLI downloaded successfully";
          setCliInstalled(true);
          loadSavedWords();
        } catch (error) {
          toast.style = Toast.Style.Failure;
          toast.title = "Failed to download Word4You CLI";
          toast.message = String(error);
          setCliInstalled(false);
        }
      } else {
        setCliInstalled(true);
        loadSavedWords();
      }
    };

    checkCliInstallation();
  }, []);

  // Auto-trigger if word is provided as argument
  useEffect(() => {
    if (initialWord && initialWord.trim() && !isLoadingSaved) {
      setSearchText(initialWord.trim());
      handleSearch(initialWord.trim());
    }
  }, [initialWord, isLoadingSaved]);

  // Clear AI result when search text changes
  useEffect(() => {
    if (aiResult && searchText.trim() !== aiResult.word) {
      setAiResult(null);
    }
  }, [searchText, aiResult]);

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

  const handleSearch = async (searchTerm: string) => {
    if (!searchTerm.trim()) {
      setAiResult(null);
      return;
    }

    const searchLower = searchTerm.toLowerCase();

    // Check if word exists locally
    const localWord = savedWordsMap.get(searchLower);

    if (localWord) {
      // Word exists locally, no need to query AI
      setAiResult(null);
      return;
    }

    // Clear previous AI result when starting a new search
    setAiResult(null);

    // Only query AI if word doesn't exist locally
    setIsLoading(true);

    const toast = await showToast({
      style: Toast.Style.Animated,
      title: `Querying "${searchTerm}"...`,
    });

    try {
      const result = await getWordExplanation(searchTerm.trim());

      if (result) {
        toast.style = Toast.Style.Success;
        toast.title = "Query completed!";
        setAiResult(result);
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to get explanation";
        toast.message = "Please check the word and try again";
        setAiResult(null);
      }
    } catch (error) {
      toast.style = Toast.Style.Failure;
      toast.title = "Error occurred";
      toast.message = String(error);
      setAiResult(null);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async (word: string, content: string) => {
    if (isSaving) return;

    setIsSaving(true);

    const toast = await showToast({
      style: Toast.Style.Animated,
      title: "Saving word to vocabulary...",
    });

    try {
      const success = await saveWordToVocabulary(word, content, (message) => {
        toast.message = message;
      });

      if (success) {
        toast.style = Toast.Style.Success;
        toast.title = "Word saved successfully!";

        // Reload saved words to include the new word
        await loadSavedWords();

        // Clear AI result since it's now saved
        setAiResult(null);
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to save word";
        toast.message = "Please check your configuration";
      }
    } catch (error) {
      toast.style = Toast.Style.Failure;
      toast.title = "Error saving word";
      toast.message = String(error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleDelete = async (word: string, timestamp?: string) => {
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: `Deleting "${word}"...`,
    });

    try {
      const success = await deleteWordFromVocabulary(word, timestamp, (message) => {
        toast.message = message;
      });

      if (success) {
        toast.style = Toast.Style.Success;
        toast.title = "Word deleted successfully!";
        await loadSavedWords();
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to delete word";
        toast.message = "Please check your configuration";
      }
    } catch (error) {
      toast.style = Toast.Style.Failure;
      toast.title = "Error deleting word";
      toast.message = String(error);
    }
  };

  const handleUpdate = async (word: string) => {
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: `Querying fresh content for "${word}"...`,
    });

    try {
      // First query the word to get fresh content
      const freshResult = await getWordExplanation(word);

      if (!freshResult) {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to get fresh content";
        toast.message = "Please check your configuration";
        return;
      }

      toast.title = `Updating "${word}"...`;

      // The CLI expects just the content without the "## word" header
      // as the word is passed separately as a parameter
      const success = await updateWordInVocabulary(word, freshResult.raw_output, undefined, (message) => {
        toast.message = message;
      });

      if (success) {
        toast.style = Toast.Style.Success;
        toast.title = "Word updated successfully!";
        await loadSavedWords();
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to update word";
        toast.message = "Please check your configuration";
      }
    } catch (error) {
      toast.style = Toast.Style.Failure;
      toast.title = "Error updating word";
      toast.message = String(error);
    }
  };

  // Filter saved words based on search text
  const filteredSavedWords = savedWords.filter(
    (word) => searchText.trim() === "" || word.word.toLowerCase().includes(searchText.toLowerCase()),
  );

  // Combine AI result with saved words
  const allWords = aiResult ? [aiResult, ...filteredSavedWords] : filteredSavedWords;

  return {
    searchText,
    setSearchText,
    savedWords,
    aiResult,
    isLoading,
    isLoadingSaved,
    isSaving,
    cliInstalled,
    filteredSavedWords,
    allWords,
    handleSearch,
    handleSave,
    handleDelete,
    handleUpdate,
  };
}
