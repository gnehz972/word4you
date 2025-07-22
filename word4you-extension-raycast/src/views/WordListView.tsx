import { List, Icon, ActionPanel, Action } from "@raycast/api";
import { WordListItem } from "../components/WordListItem";
import { WordExplanation, SavedWord } from "../types";

// Type assertion to bypass TypeScript errors with Raycast API
const ListComponent = List as any;
const ActionPanelComponent = ActionPanel as any;
const ActionComponent = Action as any;

interface WordListViewProps {
  searchText: string;
  isLoading: boolean;
  isLoadingSaved: boolean;
  allWords: (WordExplanation | SavedWord)[];
  aiResult: WordExplanation | null;
  onSearchTextChange: (text: string) => void;
  onSearch: (text: string) => void;
  onSave: (word: string, content: string) => void;
  onDelete: (word: string, timestamp?: string) => void;
  onUpdate: (word: string) => void;
}

export function WordListView({
  searchText,
  isLoading,
  isLoadingSaved,
  allWords,
  aiResult,
  onSearchTextChange,
  onSearch,
  onSave,
  onDelete,
  onUpdate,
}: WordListViewProps) {
  return (
    <ListComponent
      isLoading={isLoadingSaved || isLoading}
      searchBarPlaceholder="Search words or enter new word to query"
      onSearchTextChange={onSearchTextChange}
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
            description={
              searchText.trim()
                ? `No saved words match "${searchText}". Press Enter to query with AI.`
                : "You haven't saved any words yet. Enter a word to query with AI."
            }
            actions={
              searchText.trim() ? (
                <ActionPanelComponent>
                  <ActionComponent
                    title={`Query "${searchText}" with AI`}
                    icon="🤖"
                    onAction={() => onSearch(searchText.trim())}
                  />
                </ActionPanelComponent>
              ) : null
            }
          />
        )
      ) : (
        allWords.map((word, index) => {
          const isAiResult = aiResult && word.word === aiResult.word;

          return (
            <WordListItem
              key={`${word.word}-${isAiResult ? "ai" : "saved"}`}
              word={word}
              index={index}
              total={allWords.length}
              isAiResult={isAiResult}
              onSave={onSave}
              onDelete={onDelete}
              onUpdate={onUpdate}
            />
          );
        })
      )}
    </ListComponent>
  );
}
