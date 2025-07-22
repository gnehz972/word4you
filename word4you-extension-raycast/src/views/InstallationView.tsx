import { List, ActionPanel, Action, Icon, open, openExtensionPreferences } from "@raycast/api";

// Type assertion to bypass TypeScript errors with Raycast API
const ListComponent = List as any;
const ActionPanelComponent = ActionPanel as any;
const ActionComponent = Action as any;

export function InstallationView() {
  return (
    <ListComponent isShowingDetail>
      <ListComponent.EmptyView
        title="Word4You CLI Not Found"
        icon={Icon.Warning}
        description="Download and setup the CLI, then configure the full path in the extension preference"
        actions={
          <ActionPanelComponent>
            <ActionComponent
              title="Download Word4You CLI"
              icon={Icon.Download}
              onAction={() => open("https://github.com/gnehz972/word4you/releases")}
            />
            <ActionComponent
              title="Open Extension Preferences"
              icon={Icon.Gear}
              onAction={openExtensionPreferences}
              shortcut={{ modifiers: ["cmd"], key: "," }}
            />
          </ActionPanelComponent>
        }
      />
    </ListComponent>
  );
}
