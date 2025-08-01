import { Toast, showToast } from "@raycast/api";
import { getWordExplanation, updateWordInVocabulary } from "../services/wordService";

export function useWordUpdate(onWordUpdated?: () => Promise<void>) {
  const handleUpdate = async (word: string, existingTimestamp: string) => {
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: `Querying fresh content for "${word}"...`,
    });

    // First query the word to get fresh content
    const freshResult = await getWordExplanation(word);

    if (!freshResult) {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed to get fresh content";
      return;
    }

    toast.title = `Updating "${word}"...`;

    const success = await updateWordInVocabulary(existingTimestamp, freshResult.raw_output, (message) => {
      toast.message = message;
    });

    if (success) {
      toast.style = Toast.Style.Success;
      toast.title = "Word updated successfully!";

      if (onWordUpdated) {
        await onWordUpdated();
      }
    } else {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed to update word";
    }
  };

  return {
    handleUpdate,
  };
}
