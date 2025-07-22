import { List, ActionPanel, Action, Icon, open, openExtensionPreferences } from "@raycast/api";

export function InstallationView() {
  return (
    <List isShowingDetail>
      <List.EmptyView
        title="Word4You CLI Not Found"
        icon={Icon.Warning}
        description="Download and setup the CLI, then configure the full path in the extension preference"
        actions={
          <ActionPanel>
            <Action
              title="Download Word4you Cli"
              icon={Icon.Download}
              onAction={() => open("https://github.com/gnehz972/word4you/releases")}
            />
            <Action title="Open Extension Preferences" icon={Icon.Gear} onAction={openExtensionPreferences} />
          </ActionPanel>
        }
      />
    </List>
  );
}
