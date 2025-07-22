import { LaunchProps } from "@raycast/api";
import { Arguments } from "./types";
import { useWords } from "./hooks/useWords";
import { InstallationView } from "./views/InstallationView";
import { WordListView } from "./views/WordListView";

export default function Word4YouCommand(props: LaunchProps<{ arguments: Arguments }>) {
  const { word: argWord } = props.arguments;

  const {
    searchText,
    setSearchText,
    allWords,
    aiResult,
    isLoading,
    isLoadingSaved,
    cliInstalled,
    handleSearch,
    handleSave,
    handleDelete,
    handleUpdate,
  } = useWords(argWord);

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
