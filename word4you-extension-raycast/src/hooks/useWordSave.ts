import { useState } from "react";
import { Toast, showToast } from "@raycast/api";
import { saveWordToVocabulary } from "../services/wordService";
import { showFailureToast } from "@raycast/utils";

export function useWordSave(onWordSaved?: () => Promise<void>, onAiResultCleared?: () => void) {
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async (content: string) => {
    if (isSaving) return;

    setIsSaving(true);

    const toast = await showToast({
      style: Toast.Style.Animated,
      title: "Saving content to vocabulary...",
    });

    try {
      const success = await saveWordToVocabulary(content, (message: string) => {
        toast.message = message;
      });

      if (success) {
        toast.style = Toast.Style.Success;
        toast.title = "Content saved successfully!";

        // Reload saved words to include the new content
        if (onWordSaved) {
          await onWordSaved();
        }

        // Clear AI result since it's now saved
        if (onAiResultCleared) {
          onAiResultCleared();
        }
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to save content";
      }
    } catch (error) {
      showFailureToast(error, { title: "Failed to save content" });
    } finally {
      setIsSaving(false);
    }
  };

  return {
    isSaving,
    handleSave,
  };
}
