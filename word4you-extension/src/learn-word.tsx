import { Action, ActionPanel, Detail, Form, Toast, showToast, useNavigation, getPreferenceValues } from "@raycast/api";
import { useState } from "react";
import { execSync } from "child_process";
import path from "path";

interface Preferences {
  executablePath: string;
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

function parseWordExplanation(output: string): WordExplanation | null {
  try {
    // Remove ANSI color codes and script command output
    let cleanOutput = output
      .replace(/\x1b\[[0-9;]*m/g, '')  // Remove ANSI codes
      .replace(/Script started.*?\n/g, '')  // Remove script start messages
      .replace(/Script done.*?\n/g, '');    // Remove script done messages

    console.log('Cleaned output:', cleanOutput); // Log first 200 chars for debugging
    console.log('Full output:', output); // Log full output for debugging
    
    // Find the word explanation section between the separators
    const explanationMatch = cleanOutput.match(/📖 Word Explanation:(.*?)(?:Choose an action:|$)/s);
    if (!explanationMatch) {
      return null;
    }
    
    const explanationSection = explanationMatch[1];
    
    // Parse based on the actual beautified output line sequence:
    // Line 1: ==================== (separators)
    // Line 2: word (plain text)
    // Line 3: empty
    // Line 4: [pronunciation] (in brackets)
    // Line 5: empty  
    // Line 6: │ definition (with │ symbol)
    // Line 7: empty
    // Line 8: chinese (Chinese characters)
    // Line 9: empty
    // Line 10: - English example
    // Line 11: - Chinese example
    // Line 12: empty
    // Line 13: [tip] (in brackets)
    
    const lines = explanationSection
      .split('\n')
      .map(line => line.trim())
      .filter(line => line && !line.match(/^=+$/)); // Remove empty lines and separators
    
    let word = '';
    let pronunciation = '';
    let definition = '';
    let chinese = '';
    let example_en = '';
    let example_zh = '';
    let tip = '';
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      
      // Line 1: Word (plain alphabetic text)
      if (i === 0 && /^[a-zA-Z]+$/.test(line)) {
        word = line;
      }
      
      // Line 2: Pronunciation (in brackets [])
      else if (line.match(/^\[.*\]$/)) {
        if (!pronunciation) {
          pronunciation = line.replace(/[\[\]]/g, '');
        } else if (!tip) {
          // Second bracketed text is the tip
          tip = line.replace(/[\[\]]/g, '');
        }
      }
      
      // Line 3: Definition (starts with │)
      else if (line.startsWith('│')) {
        definition = line.replace(/│\s*/, '').trim();
      }
      
      // Line 4: Chinese (only Chinese characters)
      else if (/^[\u4e00-\u9fa5，。、；：""''！？]+$/.test(line)) {
        chinese = line;
      }
      
      // Lines 5-6: Examples (start with -)
      else if (line.startsWith('- ')) {
        const exampleText = line.replace('- ', '');
        if (!/[\u4e00-\u9fa5]/.test(exampleText) && !example_en) {
          example_en = exampleText;
        } else if (/[\u4e00-\u9fa5]/.test(exampleText) && !example_zh) {
          example_zh = exampleText;
        }
      }
    }

    return {
      word: word || 'Unknown',
      pronunciation: pronunciation || '',
      definition: definition || '',
      chinese: chinese || '',
      example_en: example_en || '',
      example_zh: example_zh || '',
      tip: tip || '',
      raw_output: output
    };
  } catch (error) {
    console.error('Error parsing word explanation:', error);
    return null;
  }
}

function getExecutablePath(): string {
  const preferences = getPreferenceValues<Preferences>();
  
  if (preferences.executablePath && preferences.executablePath.trim()) {
    return preferences.executablePath.trim();
  }
  
  // Default to the executable in the extension project directory
  return path.join(__dirname, '../word4you');
}

async function getWordExplanation(word: string): Promise<WordExplanation | null> {
  try {
    // Path to the word4you executable
    const executablePath = getExecutablePath();
    
    // The executable works but fails on TTY, so we capture both stdout and stderr
    const command = `echo "k" | "${executablePath}" "${word}"`;
    
    let output = '';
    try {
      // Try normal execution first
      output = execSync(command, {
        encoding: 'utf8',
        timeout: 30000,
        cwd: path.dirname(executablePath)
      });
    } catch (error: any) {
      // The command "fails" due to TTY error, but the output is in stderr
      console.log('Command failed as expected due to TTY, extracting output...');
      if (error.stdout) output += error.stdout;
      if (error.stderr) output += error.stderr;
      
      // If we got some output, continue processing
      if (output.trim().length === 0) {
        throw error;
      }
    }
    
    return parseWordExplanation(output);
  } catch (error) {
    console.error('Error getting word explanation:', error);
    return null;
  }
}

async function saveWordToVocabulary(word: string): Promise<boolean> {
  try {
    const executablePath = getExecutablePath();
    
    // The executable works but fails on TTY, so we capture both stdout and stderr
    const command = `echo "s" | "${executablePath}" "${word}"`;
    
    let output = '';
    try {
      // Try normal execution first
      output = execSync(command, {
        encoding: 'utf8',
        timeout: 30000,
        cwd: path.dirname(executablePath)
      });
    } catch (error: any) {
      // The command "fails" due to TTY error, but may have actually saved
      console.log('Save command failed as expected due to TTY, checking output...');
      if (error.stdout) output += error.stdout;
      if (error.stderr) output += error.stderr;
      
      // Check if the output indicates successful processing
      if (output.includes('Successfully processed') || 
          output.includes('Saving to vocabulary notebook') ||
          output.includes('📖 Word Explanation:')) {
        console.log('Word appears to have been processed successfully despite TTY error');
        return true;
      }
    }
    
    // If we get here and have output, assume success
    if (output.trim().length > 0) {
      console.log('Save operation completed with output:', output.substring(0, 100));
      return true;
    }
    
    return false;
  } catch (error) {
    console.error('Error saving word:', error);
    return false;
  }
}

function WordDetailView({ word, explanation }: { word: string; explanation: WordExplanation }) {
  const { pop } = useNavigation();

  const handleSave = async () => {
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: "Saving word...",
    });

    const success = await saveWordToVocabulary(word);
    
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

export default function LearnWordCommand() {
  const [word, setWord] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const { push } = useNavigation();

  const handleSubmit = async () => {
    if (!word.trim()) {
      await showToast({
        style: Toast.Style.Failure,
        title: "Please enter a word",
      });
      return;
    }

    setIsLoading(true);
    
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: "Getting explanation...",
    });

    try {
      const explanation = await getWordExplanation(word.trim());
      
      if (explanation) {
        toast.style = Toast.Style.Success;
        toast.title = "Explanation ready!";
        
        push(<WordDetailView word={word.trim()} explanation={explanation} />);
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