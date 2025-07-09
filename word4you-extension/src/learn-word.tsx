import { Action, ActionPanel, Detail, Form, Toast, showToast, getPreferenceValues, LaunchProps, useNavigation } from "@raycast/api";
import { useState, useEffect } from "react";
import { execSync } from "child_process";
import path from "path";
import React from "react";

interface Preferences {
  geminiApiKey: string;
  vocabularyFile: string;
  gitRemoteUrl: string;
}

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


function getExecutablePath(): string {
  // Default to the executable in the extension project directory
  return path.join(__dirname, 'assets/word4you');
}

async function getWordExplanation(word: string): Promise<WordExplanation | null> {
  try {
    const preferences = getPreferenceValues<Preferences>();
    const executablePath = getExecutablePath();
    
    // Create environment variables from preferences
    const env = {
      ...process.env,
      GEMINI_API_KEY: preferences.geminiApiKey,
      VOCABULARY_NOTEBOOK_FILE: preferences.vocabularyFile || 'vocabulary_notebook.md',
      ...(preferences.gitRemoteUrl && { GIT_REMOTE_URL: preferences.gitRemoteUrl })
    };
    
    // Use --raw flag to get clean output without TTY interaction
    const command = `"${executablePath}" --raw "${word}"`;
    
    const output = execSync(command, {
      encoding: 'utf8',
      timeout: 30000,
      cwd: path.dirname(executablePath),
      env: env
    });
    
    return parseRawWordExplanation(output, word);
  } catch (error) {
    console.error('Error getting word explanation:', error);
    return null;
  }
}

async function saveWordToVocabulary(word: string, content: string): Promise<boolean> {
  try {
    const preferences = getPreferenceValues<Preferences>();
    const executablePath = getExecutablePath();
    
    // Create environment variables from preferences
    const env = {
      ...process.env,
      GEMINI_API_KEY: preferences.geminiApiKey,
      VOCABULARY_NOTEBOOK_FILE: preferences.vocabularyFile || 'vocabulary_notebook.md',
      ...(preferences.gitRemoteUrl && { GIT_REMOTE_URL: preferences.gitRemoteUrl })
    };
    
    // Use the save command with the raw content
    const command = `"${executablePath}" save "${word}" "${content.replace(/"/g, '\\"')}"`;
    console.log('save command:', command);
    
    const output = execSync(command, {
      encoding: 'utf8',
      timeout: 30000,
      cwd: path.dirname(executablePath),
      env: env
    });
    
    // Check if the output indicates successful saving
    return output.includes('Successfully saved word') || output.includes('Saving word');
  } catch (error) {
    console.error('Error saving word:', error);
    return false;
  }
}

function WordDetailView({ word, explanation }: { word: string; explanation: WordExplanation }): JSX.Element {
  const { pop } = useNavigation();

  const handleSave = async () => {
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: "Saving word...",
    });

    const success = await saveWordToVocabulary(word, explanation.raw_output);
    
    if (success) {
      toast.style = Toast.Style.Success;
      toast.title = "Word saved!";
      toast.message = `"${word}" has been added to your vocabulary notebook`;
    } else {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed to save word";
      toast.message = "Please check your configuration";
    }
  };

  const markdown = `
# 📖 ${explanation.word}

**🔊 Pronunciation:** ${explanation.pronunciation}

**📝 Definition:** ${explanation.definition}

**🇨🇳 Chinese:** ${explanation.chinese}

## Examples
**English:** ${explanation.example_en}

**Chinese:** ${explanation.example_zh}

${explanation.tip ? `## 💡 Tip\n${explanation.tip}` : ''}
  `;

  return (
    <Detail
      markdown={markdown}
      actions={
        <ActionPanel>
          <Action
            title="Save to Vocabulary"
            icon="💾"
            onAction={handleSave}
          />
          <Action
            title="Back"
            icon="←"
            onAction={pop}
          />
        </ActionPanel>
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
      title: `Learning "${wordToLearn}"...`,
    });

    try {
      const result = await getWordExplanation(wordToLearn.trim());
      
      if (result) {
        toast.style = Toast.Style.Success;
        toast.title = "Word learned!";
        
        push(<WordDetailView word={wordToLearn.trim()} explanation={result} />);
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

  // If we have an argument word, show loading state while processing
  if (argWord && argWord.trim()) {
    return (
      <Detail
        isLoading={isLoading}
        markdown={`# 📚 Learning "${argWord}"...\n\nPlease wait while we get the explanation for "${argWord}".`}
      />
    );
  }

  // Otherwise show the input form
  return (
    <Form
      isLoading={isLoading}
      actions={
        <ActionPanel>
          <Action.SubmitForm
            title="Learn Word"
            icon="📚"
            onSubmit={handleSubmit}
          />
        </ActionPanel>
      }
    >
      <Form.TextField
        id="word"
        title="Word"
        placeholder="Enter an English word to learn"
        value={word}
        onChange={setWord}
      />
    </Form>
  );
}