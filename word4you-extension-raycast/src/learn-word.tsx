import { LaunchProps } from "@raycast/api";
import { Arguments } from "./types";
import { useWords } from "./hooks/useWords";
import { useCliSetup } from "./hooks/useCliSetup";
import { InstallationView } from "./views/InstallationView";
import { WordListView } from "./views/WordListView";

export default function Word4YouCommand(props: LaunchProps<{ arguments: Arguments }>) {
  const { word: argWord } = props.arguments;

  // CLI setup - handled directly in UI component
  const { cliInstalled } = useCliSetup();

  const {
    searchText,
    setSearchText,
    allWords,
    aiResult,
    isLoading,
    isLoadingSaved,
    handleSearch,
    handleSave,
    handleDelete,
    handleUpdate,
  } = useWords(argWord, cliInstalled);

  // If CLI is not installed, show installation instructions
  if (cliInstalled === false) {
    return <InstallationView />;
  }

  return (
    <WordListView
      searchText={searchText}
      isLoading={isLoading}
      isLoadingSaved={isLoadingSaved}
      allWords={allWords}
      aiResult={aiResult}
      onSearchTextChange={setSearchText}
      onSearch={handleSearch}
      onSave={handleSave}
      onDelete={handleDelete}
      onUpdate={handleUpdate}
    />
  );
}
