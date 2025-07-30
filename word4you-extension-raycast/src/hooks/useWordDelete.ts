import { Toast, showToast } from "@raycast/api";
import { deleteWordFromVocabulary } from "../services/wordService";
import { showFailureToast } from "@raycast/utils";

export function useWordDelete(onWordDeleted?: () => Promise<void>) {
  const handleDelete = async (timestamp: string) => {
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: `Deleting entry...`,
    });

    try {
      const success = await deleteWordFromVocabulary(timestamp, (message: string) => {
        toast.message = message;
      });

      if (success) {
        toast.style = Toast.Style.Success;
        toast.title = "Entry deleted successfully!";

        if (onWordDeleted) {
          await onWordDeleted();
        }
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to delete entry";
      }
    } catch (error) {
      showFailureToast(error, { title: "Failed to delete entry" });
    }
  };

  return {
    handleDelete,
  };
}
