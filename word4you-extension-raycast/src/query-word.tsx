import { Action, ActionPanel, Detail, List, Toast, showToast, LaunchProps, useNavigation } from "@raycast/api";
import { useState, useEffect } from "react";
import { execSync, spawn } from "child_process";
import { createInterface } from "readline";
import React from "react";
import { getPreferences, getVocabularyPath, ensureVocabularyDirectoryExists, getExecutablePath, createEnvironmentFromPreferences } from "./config";

// Type assertion to bypass TypeScript errors with Raycast API
const DetailComponent = Detail as any;
const ListComponent = List as any;
const ActionPanelComponent = ActionPanel as any;
const ActionComponent = Action as any;

interface Arguments {
  word: string;
}

interface WordExplanation {
  word: string;
  pronunciation: string;
  definition: string;
  chinese: string;
  example_en: string;
  example_zh: string;
  tip: string;
  raw_output: string;
}

function parseRawWordExplanation(output: string, word: string): WordExplanation | null {
  try {
    // Raw output format:
    // ## word
    // */pronunciation/*
    // > Definition
    // **Chinese**
    // - English example
    // - Chinese example
    // *Tip*
    
    const lines = output.split('\n').map(line => line.trim()).filter(line => line);
    
    let pronunciation = '';
    let definition = '';
    let chinese = '';
    let example_en = '';
    let example_zh = '';
    let tip = '';
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      
      // Pronunciation: */pronunciation/*
      if (line.match(/^\*\/.*\/\*$/)) {
        pronunciation = line.replace(/^\*\//, '').replace(/\/\*$/, '');
      }
      
      // Definition: > Definition text
      else if (line.startsWith('> ')) {
        definition = line.replace(/^> /, '');
      }
      
      // Chinese: **Chinese text**
      else if (line.match(/^\*\*.*\*\*$/)) {
        chinese = line.replace(/^\*\*/, '').replace(/\*\*$/, '');
      }
      
      // Examples: - Example text
      else if (line.startsWith('- ')) {
        const exampleText = line.replace(/^- /, '');
        if (!/[\u4e00-\u9fa5]/.test(exampleText) && !example_en) {
          example_en = exampleText;
        } else if (/[\u4e00-\u9fa5]/.test(exampleText) && !example_zh) {
          example_zh = exampleText;
        }
      }
      
      // Tip: *Tip text* (but not pronunciation format)
      else if (line.match(/^\*.*\*$/) && !line.match(/^\*\/.*\/\*$/)) {
        tip = line.replace(/^\*/, '').replace(/\*$/, '');
      }
    }
    
    return {
      word: word,
      pronunciation: pronunciation || '',
      definition: definition || '',
      chinese: chinese || '',
      example_en: example_en || '',
      example_zh: example_zh || '',
      tip: tip || '',
      raw_output: output
    };
  } catch (error) {
    console.error('Error parsing raw word explanation:', error);
    return null;
  }
}


async function getWordExplanation(word: string): Promise<WordExplanation | null> {
  try {
    const preferences = getPreferences();
    const executablePath = getExecutablePath();
    
    // Use cross-platform path resolution for vocabulary file
    const vocabularyPath = getVocabularyPath(preferences.vocabularyBaseDir);
    
    // Ensure the directory exists
    ensureVocabularyDirectoryExists(vocabularyPath);
    
    // Create environment variables from preferences
    const env = createEnvironmentFromPreferences();
    
    // Use --raw flag to get clean output without TTY interaction
    const command = `"${executablePath}" --raw "${word}"`;
    
    const output = execSync(command, {
      encoding: 'utf8',
      timeout: 30000,
      cwd: require('path').dirname(executablePath),
      env: env
    });
    
    return parseRawWordExplanation(output, word);
  } catch (error) {
    console.error('Error getting word explanation:', error);
    return null;
  }
}

async function saveWordToVocabulary(word: string, content: string, onStatusUpdate?: (message: string) => void): Promise<boolean> {
  return new Promise((resolve) => {
    try {
      const preferences = getPreferences();
      const executablePath = getExecutablePath();
      
      // Use cross-platform path resolution for vocabulary file
      const vocabularyPath = getVocabularyPath(preferences.vocabularyBaseDir);
      
      // Ensure the directory exists
      ensureVocabularyDirectoryExists(vocabularyPath);
      
      // Create environment variables from preferences
      const env = createEnvironmentFromPreferences();
      
      // Use spawn to capture real-time output
      const child = spawn(executablePath, ['save', word, content], {
        cwd: require('path').dirname(executablePath),
        env: env,
        stdio: ['pipe', 'pipe', 'pipe']
      });
      
      let fullOutput = '';
      let success = false;
      
      // Use readline for perfect line buffering
      const rl = createInterface({
        input: child.stdout,
        crlfDelay: Infinity
      });
      
      // Process each complete line as it arrives
      rl.on('line', (line) => {
        const trimmedLine = line.trim();
        if (trimmedLine) {
          fullOutput += trimmedLine + '\n';
          if (onStatusUpdate) {
            onStatusUpdate(trimmedLine);
          }
        }
      });
      
      // Capture stderr
      child.stderr.on('data', (data) => {
        console.error('stderr:', data.toString());
      });
      
      // Handle process completion
      child.on('close', (code) => {
        rl.close();
        success = fullOutput.includes('Successfully saved word') || fullOutput.includes('Saving word');
        
        // Add a small delay to ensure final summary toast is visible
        setTimeout(() => {
          if (code === 0 && success) {
            resolve(true);
          } else {
            resolve(false);
          }
        }, 500);
      });
      
      // Handle errors
      child.on('error', (error) => {
        console.error('Error saving word:', error);
        rl.close();
        resolve(false);
      });
      
      // Set timeout
      setTimeout(() => {
        child.kill();
        rl.close();
        resolve(false);
      }, 30000);
      
    } catch (error) {
      console.error('Error saving word:', error);
      resolve(false);
    }
  });
}

function WordDetailView({ word, explanation }: { word: string; explanation: WordExplanation }): JSX.Element {
  const { pop } = useNavigation();
  const [isSaving, setIsSaving] = useState(false);
  const [isSaved, setIsSaved] = useState(false);

  const handleSave = async () => {
    if (isSaving) return; // Prevent duplicate saves
    
    setIsSaving(true);
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: "Saving word...",
    });

    // Track the status of each step
    let savedLocally = false;
    let committedLocally = false;
    let pushedToRemote = false;

    const success = await saveWordToVocabulary(word, explanation.raw_output, (statusMessage) => {
      // Update toast with real-time status messages
      toast.title = statusMessage;
      
      // Track completion of each step
      if (statusMessage.includes('Successfully saved word locally')) {
        savedLocally = true;
      } else if (statusMessage.includes('Successfully committed word locally')) {
        committedLocally = true;
      } else if (statusMessage.includes('Successfully pushed word to remote')) {
        pushedToRemote = true;
      }
    });
    
    if (success) {
      // Show summary toast with all steps
      const summaryMessage = [
        savedLocally ? '✅ Saved locally' : '❌ Save failed',
        committedLocally ? '✅ Committed locally' : '❌ Commit failed',
        pushedToRemote ? '✅ Pushed to remote' : '⚠️ No remote push'
      ].join('\n');
      
      toast.style = Toast.Style.Success;
      toast.title = summaryMessage;
      setIsSaved(true); // Mark as saved permanently
    } else {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed to save word";
      toast.message = "Please check your configuration";
    }
    
    setIsSaving(false);
  };

  // Helper: render a register/usage if present in tip
  function renderRegister(text: string | undefined) {
    if (!text) return '';
    if (/^(informal|formal|slang|archaic|literary|technical)$/i.test(text.trim())) {
      return `_${text}_\n`;
    }
    return '';
  }

  const markdown = `
# ${explanation.word}
${explanation.pronunciation ? `\n*/${explanation.pronunciation}/*` : ''}
${explanation.definition ? `\n*${explanation.definition}*` : ''}
${explanation.chinese ? `\n*${explanation.chinese}*` : ''}
${explanation.example_en ? `\n> _${explanation.example_en}_` : ''}
${explanation.example_zh ? `\n> _${explanation.example_zh}_` : ''}
${explanation.tip ? `\n💡*${explanation.tip}*` : ''}
`;

  return (
    <DetailComponent
      markdown={markdown}
      actions={
        <ActionPanelComponent>
          {!isSaved && !isSaving && (
            <ActionComponent
              title="Save to Vocabulary"
              icon="💾"
              onAction={handleSave}
            />
          )}
          {!isSaved && isSaving && (
            <ActionComponent
              title="Saving..."
              icon="⏳"
              onAction={() => {}} // No-op action while saving
            />
          )}
          {isSaved && (
            <ActionComponent
              title="Already Saved"
              icon="✅"
              onAction={pop} // Navigate back to main page
            />
          )}
          <ActionComponent
            title="Close"
            icon="✖️"
            onAction={pop}
            shortcut={{ modifiers: ["opt"], key: "escape" }}
          />
        </ActionPanelComponent>
      }
    />
  );
}

export default function LearnWordCommand(props: LaunchProps<{ arguments: Arguments }>): JSX.Element {
  const { word: argWord } = props.arguments;
  const [word, setWord] = useState(argWord || "");
  const [isLoading, setIsLoading] = useState(false);
  const [explanation, setExplanation] = useState<WordExplanation | null>(null);
  const { push } = useNavigation();

  const handleLearnWord = async (wordToLearn: string) => {
    if (!wordToLearn.trim()) {
      await showToast({
        style: Toast.Style.Failure,
        title: "Please enter a word",
      });
      return;
    }

    setIsLoading(true);
    
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: `Querying "${wordToLearn}"...`,
    });

    try {
      const result = await getWordExplanation(wordToLearn.trim());
      
      if (result) {
        toast.style = Toast.Style.Success;
        toast.title = "Query completed!";
        
        // If this was triggered by an argument, set the explanation directly
        // so we can render the detail view immediately
        if (argWord && argWord.trim() === wordToLearn.trim()) {
          setExplanation(result);
        } else {
                  // Otherwise, push to the detail view
        push(<WordDetailView word={wordToLearn.trim()} explanation={result} /> as any);
        }
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to get explanation";
        toast.message = "Please check the word and try again";
      }
    } catch (error) {
      toast.style = Toast.Style.Failure;
      toast.title = "Error occurred";
      toast.message = String(error);
    } finally {
      setIsLoading(false);
    }
  };

  // Auto-trigger if word is provided as argument
  useEffect(() => {
    if (argWord && argWord.trim()) {
      handleLearnWord(argWord);
    }
  }, [argWord]);

  const handleSubmit = async () => {
    await handleLearnWord(word);
  };

  // If we have an argument word and we're still loading, show loading state
  if (argWord && argWord.trim() && isLoading) {
    return (
      <DetailComponent
        isLoading={isLoading}
        markdown={`# 📚 Querying "${argWord}"...\n\nPlease wait while we get the explanation for "${argWord}".`}
      />
    );
  }

  // If we have an argument word and we have the explanation, show the detail view directly
  if (argWord && argWord.trim() && explanation) {
    return <WordDetailView word={argWord.trim()} explanation={explanation} />;
  }

  // Show loading state if loading
  if (isLoading) {
    return (
      <DetailComponent
        isLoading={isLoading}
        markdown={`# 📚 Querying "${word}"...\n\nPlease wait while we get the explanation for "${word}".`}
      />
    );
  }

  // Otherwise show the input field
  return (
    <ListComponent
      searchBarPlaceholder="Enter an English word to query"
      onSearchTextChange={setWord}
      searchText={word}
      actions={
        <ActionPanelComponent>
          <ActionComponent
            title="Query Word"
            icon="📚"
            onAction={() => handleSubmit()}
            shortcut={{ modifiers: [], key: "return" }}
          />
        </ActionPanelComponent>
      }
    >
      {word.trim() && (
        <ListComponent.Item
          title={`Query "${word}"`}
          subtitle="Press Enter to query this word"
          icon="📚"
          actions={
            <ActionPanelComponent>
              <ActionComponent
                title="Query Word"
                icon="📚"
                onAction={() => handleSubmit()}
                shortcut={{ modifiers: [], key: "return" }}
              />
            </ActionPanelComponent>
          }
        />
      )}
    </ListComponent>
  );
}