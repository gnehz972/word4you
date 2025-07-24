import { useState, useEffect } from "react";
import { Toast, showToast } from "@raycast/api";
import { isCliInstalled, ensureCLI } from "../services/cliManager";

export function useCliSetup() {
  const [cliInstalled, setCliInstalled] = useState<boolean | null>(null);

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
        } catch (error) {
          toast.style = Toast.Style.Failure;
          toast.title = "Failed to download Word4You CLI";
          toast.message = String(error);
          setCliInstalled(false);
        }
      } else {
        setCliInstalled(true);
      }
    };

    checkCliInstallation();
  }, []);

  return {
    cliInstalled,
  };
}