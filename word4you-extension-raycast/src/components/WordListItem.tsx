import { List, ActionPanel, Action } from "@raycast/api";
import { WordExplanation, SavedWord } from "../types";
import { WordDetail } from "./WordDetail";

// Type assertion to bypass TypeScript errors with Raycast API
const ListComponent = List as any;
const ActionPanelComponent = ActionPanel as any;
const ActionComponent = Action as any;

interface WordListItemProps {
  word: WordExplanation | SavedWord;
  index?: number;
  total?: number;
  isAiResult?: boolean;
  onSave?: (word: string, content: string) => void;
  onDelete?: (word: string, timestamp?: string) => void;
  onUpdate?: (word: string) => void;
}

export function WordListItem({
  word,
  index,
  total,
  isAiResult = false,
  onSave,
  onDelete,
  onUpdate,
}: WordListItemProps) {
  return (
    <ListComponent.Item
      title={word.word}
      subtitle={word.chinese}
      accessories={[isAiResult ? { text: "AI Result" } : { text: `${index! + 1} of ${total}` }]}
      detail={<WordDetail word={word} />}
      actions={
        <ActionPanelComponent>
          {isAiResult && onSave && (
            <ActionComponent title="Save to Vocabulary" icon="💾" onAction={() => onSave(word.word, word.raw_output)} />
          )}
          {!isAiResult && (
            <>
              {onDelete && (
                <ActionComponent
                  title="Delete Word"
                  icon="🗑️"
                  onAction={() => onDelete(word.word, (word as SavedWord).timestamp)}
                />
              )}
              {onUpdate && <ActionComponent title="Update Word" icon="📝" onAction={() => onUpdate(word.word)} />}
            </>
          )}
        </ActionPanelComponent>
      }
    />
  );
}
