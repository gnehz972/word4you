import { Action, ActionPanel, List, Toast, showToast, LaunchProps, useNavigation, Icon } from "@raycast/api";
import { useState, useEffect } from "react";
import { execSync, spawn } from "child_process";
import React from "react";
import fs from "fs";
import { getPreferences, getVocabularyPath, ensureVocabularyDirectoryExists, getExecutablePath, createEnvironmentFromPreferences } from "./config";

// Type assertion to bypass TypeScript errors with Raycast API
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

interface SavedWord {
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
      
      child.stdout.on('data', (data) => {
        const message = data.toString();
        fullOutput += message;
        if (onStatusUpdate) {
          onStatusUpdate(message.trim());
        }
      });
      
      child.stderr.on('data', (data) => {
        const message = data.toString();
        fullOutput += message;
        if (onStatusUpdate) {
          onStatusUpdate(message.trim());
        }
      });
      
      child.on('close', (code) => {
        success = code === 0;
        resolve(success);
      });
      
      child.on('error', (error) => {
        console.error('Error spawning process:', error);
        resolve(false);
      });
      
    } catch (error) {
      console.error('Error in saveWordToVocabulary:', error);
      resolve(false);
    }
  });
}

// Parse saved words from the vocabulary notebook
function parseSavedWords(vocabularyPath: string): SavedWord[] {
  try {
    if (!fs.existsSync(vocabularyPath)) {
      return [];
    }

    const content = fs.readFileSync(vocabularyPath, 'utf8');
    const words: SavedWord[] = [];
    
    // Split content by word sections (## word)
    const sections = content.split(/(?=^## )/m);
    
    for (const section of sections) {
      if (!section.trim()) continue;
      
      const lines = section.split('\n');
      const wordLine = lines[0];
      const wordMatch = wordLine.match(/^## (.+)$/);
      
      if (!wordMatch) continue;
      
      const word = wordMatch[1].trim();
      const wordContent = lines.slice(1).join('\n');
      
      // Parse the word content similar to the original parser
      const parsed = parseRawWordExplanation(wordContent, word);
      if (parsed) {
        words.push(parsed);
      }
    }
    
    return words;
  } catch (error) {
    console.error('Error parsing saved words:', error);
    return [];
  }
}

export default function Word4YouCommand(props: LaunchProps<{ arguments: Arguments }>): JSX.Element {
  const { word: argWord } = props.arguments;
  const [searchText, setSearchText] = useState(argWord || "");
  const [savedWords, setSavedWords] = useState<SavedWord[]>([]);
  const [aiResult, setAiResult] = useState<WordExplanation | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isLoadingSaved, setIsLoadingSaved] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [savedWordsMap, setSavedWordsMap] = useState<Map<string, SavedWord>>(new Map());

  useEffect(() => {
    loadSavedWords();
  }, []);

  // Auto-trigger if word is provided as argument
  useEffect(() => {
    if (argWord && argWord.trim()) {
      handleSearch(argWord.trim());
    }
  }, [argWord, savedWordsMap]);

  const loadSavedWords = async () => {
    try {
      const preferences = getPreferences();
      const vocabularyPath = getVocabularyPath(preferences.vocabularyBaseDir);
      const words = parseSavedWords(vocabularyPath);
      setSavedWords(words);
      
      // Create a map for quick lookup
      const wordsMap = new Map<string, SavedWord>();
      words.forEach(word => wordsMap.set(word.word.toLowerCase(), word));
      setSavedWordsMap(wordsMap);
    } catch (error) {
      console.error('Error loading saved words:', error);
      await showToast({
        style: Toast.Style.Failure,
        title: "Error",
        message: "Failed to load saved words"
      });
    } finally {
      setIsLoadingSaved(false);
    }
  };

  const handleSearch = async (searchTerm: string) => {
    if (!searchTerm.trim()) {
      setAiResult(null);
      return;
    }

    const searchLower = searchTerm.toLowerCase();
    
    // Check if word exists locally
    const localWord = savedWordsMap.get(searchLower);
    
    if (localWord) {
      // Word exists locally, no need to query AI
      setAiResult(null);
      return;
    }

    // Only query AI if word doesn't exist locally
    setIsLoading(true);
    
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: `Querying "${searchTerm}"...`,
    });

    try {
      const result = await getWordExplanation(searchTerm.trim());
      
      if (result) {
        toast.style = Toast.Style.Success;
        toast.title = "Query completed!";
        setAiResult(result);
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to get explanation";
        toast.message = "Please check the word and try again";
        setAiResult(null);
      }
    } catch (error) {
      toast.style = Toast.Style.Failure;
      toast.title = "Error occurred";
      toast.message = String(error);
      setAiResult(null);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async (word: string, content: string) => {
    if (isSaving) return;
    
    setIsSaving(true);
    
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: "Saving word to vocabulary...",
    });

    try {
      const success = await saveWordToVocabulary(word, content, (message) => {
        toast.message = message;
      });
      
      if (success) {
        toast.style = Toast.Style.Success;
        toast.title = "Word saved successfully!";
        
        // Reload saved words to include the new word
        await loadSavedWords();
        
        // Clear AI result since it's now saved
        setAiResult(null);
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Failed to save word";
        toast.message = "Please check your configuration";
      }
    } catch (error) {
      toast.style = Toast.Style.Failure;
      toast.title = "Error saving word";
      toast.message = String(error);
    } finally {
      setIsSaving(false);
    }
  };

  // Filter saved words based on search text
  const filteredSavedWords = savedWords.filter(word => 
    searchText.trim() === '' || 
    word.word.toLowerCase().includes(searchText.toLowerCase())
  );

  // Combine AI result with saved words
  const allWords = aiResult ? [aiResult, ...filteredSavedWords] : filteredSavedWords;

  return (
    <ListComponent 
      isLoading={isLoadingSaved || isLoading}
      searchBarPlaceholder="Search words or enter new word to query"
      onSearchTextChange={setSearchText}
      searchText={searchText}
      isShowingDetail
    >
      {allWords.length === 0 ? (
        isLoading ? (
          <ListComponent.EmptyView
            title="Querying..."
            icon={Icon.Cloud}
            description="Please wait while we query the word..."
          />
        ) : (
          <ListComponent.EmptyView
            title="No Words Found"
            description={searchText.trim() ? 
              `No saved words match "${searchText}". Press Enter to query with AI.` :
              "You haven't saved any words yet. Enter a word to query with AI."
            }
            actions={
              searchText.trim() ? (
                <ActionPanelComponent>
                  <ActionComponent
                    title={`Query "${searchText}" with AI`}
                    icon="🤖"
                    onAction={() => handleSearch(searchText.trim())}
                  />
                  <ActionComponent
                    title="Refresh Word List"
                    shortcut={{ modifiers: ["cmd"], key: "r" }}
                    onAction={loadSavedWords}
                  />
                </ActionPanelComponent>
              ) : (
                <ActionPanelComponent>
                  <ActionComponent
                    title="Refresh Word List"
                    shortcut={{ modifiers: ["cmd"], key: "r" }}
                    onAction={loadSavedWords}
                  />
                </ActionPanelComponent>
              )
            }
          />
        )
      ) : (
        allWords.map((word, index) => {
          const isAiResult = aiResult && word.word === aiResult.word;
          const markdown = `
# ${word.word}
${word.pronunciation ? `\n*/${word.pronunciation}/*` : ''}
${word.definition ? `\n*${word.definition}*` : ''}
${word.chinese ? `\n*${word.chinese}*` : ''}
${word.example_en ? `\n> _${word.example_en}_` : ''}
${word.example_zh ? `\n> _${word.example_zh}_` : ''}
${word.tip ? `\n💡*${word.tip}*` : ''}
`;

          return (
            <ListComponent.Item
              key={`${word.word}-${isAiResult ? 'ai' : 'saved'}`}
              title={word.word}
              subtitle={word.chinese}
              accessories={[
                isAiResult ? { text: "AI Result" } : { text: `${index + 1} of ${allWords.length}` }
              ]}
              detail={
                <ListComponent.Item.Detail markdown={markdown} />
              }
              actions={
                <ActionPanelComponent>
                  {isAiResult && (
                    <ActionComponent
                      title="Save to Vocabulary"
                      icon="💾"
                      onAction={() => handleSave(word.word, word.raw_output)}
                    />
                  )}
                  <ActionComponent
                    title="Refresh Word List"
                    shortcut={{ modifiers: ["cmd"], key: "r" }}
                    onAction={loadSavedWords}
                  />
                </ActionPanelComponent>
              }
            />
          );
        })
      )}
    </ListComponent>
  );
} 