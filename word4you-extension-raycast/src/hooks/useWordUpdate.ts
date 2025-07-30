import { Toast, showToast } from "@raycast/api";
import { getWordExplanation, updateWordInVocabulary } from "../services/wordService";
import { showFailureToast } from "@raycast/utils";

export function useWordUpdate(onWordUpdated?: () => Promise<void>) {
  const handleUpdate = async (word: string, existingTimestamp?: string) => {
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

      // The CLI now requires a timestamp to identify which entry to update
      // We need the timestamp from the existing saved word entry
      if (!existingTimestamp) {
        toast.style = Toast.Style.Failure;
        toast.title = "Cannot update word";
        toast.message = "No timestamp provided for existing entry";
        return;
      }

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
    } catch (error) {
      showFailureToast(error, { title: "Error updating word" });
    }
  };

  return {
    handleUpdate,
  };
}
