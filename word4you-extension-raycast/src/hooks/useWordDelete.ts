import { Toast, showToast } from "@raycast/api";
import { deleteWordFromVocabulary } from "../services/wordService";

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
      toast.style = Toast.Style.Failure;
      toast.title = "Error deleting entry";
      toast.message = error instanceof Error ? error.message : "An unknown error occurred";
      console.error("Error details:", error);
    }
  };

  return {
    handleDelete,
  };
}
