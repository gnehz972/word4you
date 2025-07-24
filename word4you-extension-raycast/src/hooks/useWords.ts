import { useSavedWords } from "./useSavedWords";
import { useWordSearch } from "./useWordSearch";
import { useWordFiltering } from "./useWordFiltering";
import { useWordSave } from "./useWordSave";
import { useWordDelete } from "./useWordDelete";
import { useWordUpdate } from "./useWordUpdate";

export function useWords(initialWord?: string, cliInstalled?: boolean) {
  // Saved words management
  const { savedWords, isLoadingSaved, savedWordsMap, loadSavedWords } = useSavedWords(cliInstalled);

  // Search functionality
  const { searchText, setSearchText, aiResult, isLoading, handleSearch, clearAiResult } = useWordSearch(
    savedWordsMap,
    isLoadingSaved,
    initialWord
  );

  // Word filtering
  const { filteredSavedWords, allWords } = useWordFiltering(savedWords, aiResult, searchText);

  // Word operations
  const { isSaving, handleSave } = useWordSave(loadSavedWords, clearAiResult);
  const { handleDelete } = useWordDelete(loadSavedWords);
  const { handleUpdate } = useWordUpdate(loadSavedWords);

  return {
    searchText,
    setSearchText,
    savedWords,
    aiResult,
    isLoading,
    isLoadingSaved,
    isSaving,
    filteredSavedWords,
    allWords,
    handleSearch,
    handleSave,
    handleDelete,
    handleUpdate,
  };
}
