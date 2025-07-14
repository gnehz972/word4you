import { Action, ActionPanel, List, Toast, showToast, LaunchProps, useNavigation } from "@raycast/api";
import { useState, useEffect } from "react";
import { execSync } from "child_process";
import React from "react";
import fs from "fs";
import { getPreferences, getVocabularyPath } from "./config";

// Type assertion to bypass TypeScript errors with Raycast API
const ListComponent = List as any;
const ActionPanelComponent = ActionPanel as any;
const ActionComponent = Action as any;

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

function parseRawWordExplanation(output: string, word: string): SavedWord | null {
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



export default function LearnWordCommand(): JSX.Element {
  const [savedWords, setSavedWords] = useState<SavedWord[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadSavedWords();
  }, []);

  const loadSavedWords = async () => {
    try {
      const preferences = getPreferences();
      const vocabularyPath = getVocabularyPath(preferences.vocabularyBaseDir);
      const words = parseSavedWords(vocabularyPath);
      setSavedWords(words);
    } catch (error) {
      console.error('Error loading saved words:', error);
      await showToast({
        style: Toast.Style.Failure,
        title: "Error",
        message: "Failed to load saved words"
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <ListComponent isLoading={isLoading} isShowingDetail>
      {savedWords.length === 0 ? (
        <ListComponent.EmptyView
          title="No Saved Words"
          description="You haven't saved any words yet. Use the Query Word command to add words to your vocabulary."
        />
      ) : (
        savedWords.map((word, index) => {
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
              key={word.word}
              title={word.word}
              subtitle={word.pronunciation || word.definition}
              accessories={[
                { text: `${index + 1} of ${savedWords.length}` }
              ]}
              detail={
                <ListComponent.Item.Detail markdown={markdown} />
              }
              actions={
                <ActionPanelComponent>
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