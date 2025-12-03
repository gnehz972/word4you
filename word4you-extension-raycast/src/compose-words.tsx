import { Detail, ActionPanel, Action, showToast, Toast, List, Icon } from "@raycast/api";
import { useState, useEffect, useCallback } from "react";
import { useCliSetup } from "./hooks/useCliSetup";
import { useSavedMdDefinitions } from "./hooks/useSavedMdDefinitions";
import { InstallationView } from "./views/InstallationView";
import { ProviderSetupView } from "./views/ProviderSetupView";
import { isProviderConfigured } from "./config";
import { executeWordCli } from "./services/cliManager";
import { saveMdDefinitionToVocabulary } from "./services/mdDefinitionService";

interface ComposedSentence {
  english: string;
  chinese: string;
  word1: string;
  word2: string;
}

function parseComposedSentence(output: string, word1: string, word2: string): ComposedSentence | null {
  try {
    const lines = output
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line);

    let english = "";
    let chinese = "";

    for (const line of lines) {
      // Skip the header line: ## word1 + word2
      if (line.startsWith("## ") && line.includes("+")) {
        continue;
      }
      // English sentence: - [SENTENCE] (first bullet without Chinese)
      if (line.startsWith("- ") && !english) {
        const text = line.replace(/^- /, "");
        // Check if it's English (no Chinese characters)
        if (!/[\u4e00-\u9fa5]/.test(text)) {
          english = text;
        }
      }
      // Chinese translation: - [中文翻译] (bullet with Chinese)
      else if (line.startsWith("- ") && english && !chinese) {
        const text = line.replace(/^- /, "");
        // Check if it contains Chinese characters
        if (/[\u4e00-\u9fa5]/.test(text)) {
          chinese = text;
        }
      }
    }

    if (english && chinese) {
      return { english, chinese, word1, word2 };
    }

    return null;
  } catch (error) {
    console.error("Error parsing composed sentence:", error);
    return null;
  }
}

function getRandomWords(words: string[], count: number): string[] {
  if (words.length < count) {
    return words;
  }

  const shuffled = [...words].sort(() => Math.random() - 0.5);
  return shuffled.slice(0, count);
}

function ComposeWordsView() {
  const { savedMdDefinitions, isLoadingSaved } = useSavedMdDefinitions();
  const [composedSentence, setComposedSentence] = useState<ComposedSentence | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentWords, setCurrentWords] = useState<{ word1: string; word2: string } | null>(null);

  const generateSentence = useCallback(
    async (useExistingWords = false) => {
      // Filter out composed sentences (those with " + " in the title)
      const singleWords = savedMdDefinitions.filter((def) => !def.text.includes(" + "));

      if (singleWords.length < 2) {
        setError("You need at least 2 saved words to compose. Please save more words first.");
        return;
      }

      setIsGenerating(true);
      setError(null);
      setComposedSentence(null);

      try {
        let word1: string;
        let word2: string;

        if (useExistingWords && currentWords) {
          // Use the same words
          word1 = currentWords.word1;
          word2 = currentWords.word2;
        } else {
          // Get all saved word texts (excluding composed sentences)
          const allWords = singleWords.map((def) => def.text);
          // Pick 2 random words
          [word1, word2] = getRandomWords(allWords, 2);
          setCurrentWords({ word1, word2 });
        }

        await showToast({
          style: Toast.Style.Animated,
          title: "Generating sentence...",
          message: `Composing: ${word1} + ${word2}`,
        });

        // Call CLI to compose sentence
        const output = await executeWordCli(["compose", word1, word2]);

        const parsed = parseComposedSentence(output, word1, word2);

        if (parsed) {
          setComposedSentence(parsed);
          await showToast({
            style: Toast.Style.Success,
            title: "Sentence generated!",
          });
        } else {
          setError("Failed to parse the generated sentence. Please try again.");
          await showToast({
            style: Toast.Style.Failure,
            title: "Failed to parse sentence",
          });
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : "Unknown error occurred";
        setError(errorMessage);
        await showToast({
          style: Toast.Style.Failure,
          title: "Failed to generate sentence",
          message: errorMessage,
        });
      } finally {
        setIsGenerating(false);
      }
    },
    [savedMdDefinitions, currentWords],
  );

  // Filter out composed sentences for counting
  const singleWordsCount = savedMdDefinitions.filter((def) => !def.text.includes(" + ")).length;

  // Auto-generate on first load when words are available
  useEffect(() => {
    if (!isLoadingSaved && singleWordsCount >= 2 && !composedSentence && !isGenerating && !error) {
      generateSentence(false);
    }
  }, [isLoadingSaved, singleWordsCount, composedSentence, isGenerating, error, generateSentence]);

  if (isLoadingSaved) {
    return <List isLoading={true} />;
  }

  if (singleWordsCount < 2) {
    return (
      <Detail
        markdown={`# Not Enough Words

You need at least **2 saved words** to compose sentences.

Currently you have **${singleWordsCount}** saved word(s) (excluding composed sentences).

Please use the **Learn Word** command to save more words to your vocabulary notebook first.`}
        actions={
          <ActionPanel>
            <Action.Push title="Learn Words" icon={Icon.Book} target={<Detail markdown="Use the Learn Word command" />} />
          </ActionPanel>
        }
      />
    );
  }

  if (isGenerating) {
    const loadingMsg = currentWords
      ? `# Composing ${currentWords.word1} + ${currentWords.word2}...`
      : "# Generating sentence...";
    return <Detail isLoading={true} markdown={loadingMsg} />;
  }

  if (error) {
    return (
      <Detail
        markdown={`# Error

${error}`}
        actions={
          <ActionPanel>
            <Action title="Try Again" icon={Icon.RotateClockwise} onAction={generateSentence} />
          </ActionPanel>
        }
      />
    );
  }

  if (composedSentence) {
    const markdown = `
# ${composedSentence.word1} + ${composedSentence.word2}

> _${composedSentence.english}_

> _${composedSentence.chinese}_
`;

    const handleSave = async () => {
      const content = `## ${composedSentence.word1} + ${composedSentence.word2}

- ${composedSentence.english}

- ${composedSentence.chinese}`;

      await showToast({
        style: Toast.Style.Animated,
        title: "Saving sentence...",
      });

      const success = await saveMdDefinitionToVocabulary(content);

      if (success) {
        await showToast({
          style: Toast.Style.Success,
          title: "Sentence saved!",
        });
      } else {
        await showToast({
          style: Toast.Style.Failure,
          title: "Failed to save sentence",
        });
      }
    };

    return (
      <Detail
        markdown={markdown}
        actions={
          <ActionPanel>
            <Action
              title="Save Sentence"
              icon={Icon.SaveDocument}
              onAction={handleSave}
              shortcut={{ modifiers: ["cmd"], key: "s" }}
            />
            <Action
              title="Regenerate Sentence"
              icon={Icon.RotateClockwise}
              onAction={() => generateSentence(true)}
              shortcut={{ modifiers: ["cmd"], key: "r" }}
            />
            <Action
              title="Generate with New Words"
              icon={Icon.Shuffle}
              onAction={() => generateSentence(false)}
              shortcut={{ modifiers: ["cmd", "shift"], key: "r" }}
            />
            <Action.CopyToClipboard
              title="Copy English Sentence"
              content={composedSentence.english}
              shortcut={{ modifiers: ["cmd"], key: "c" }}
            />
            <Action.CopyToClipboard
              title="Copy Chinese Translation"
              content={composedSentence.chinese}
              shortcut={{ modifiers: ["cmd", "shift"], key: "c" }}
            />
          </ActionPanel>
        }
      />
    );
  }

  return <Detail markdown="# Loading..." />;
}

export default function ComposeWordsCommand() {
  const { cliInstalled } = useCliSetup();

  // Show provider setup view if user hasn't configured their AI provider and API key
  if (!isProviderConfigured()) {
    return <ProviderSetupView />;
  }

  if (cliInstalled === undefined) {
    return <List isLoading={true} />;
  }

  if (!cliInstalled) {
    return <InstallationView />;
  }

  return <ComposeWordsView />;
}

