import type {
  WorkEnvironmentChanged,
  WorkChanged,
  WorkHumanChanged,
  WorkDecisionPreferenceChanged,
  WorkModelsChanged,
  AccountChanged,
  PanelState,
  PageAssistChanged,
  ProfilesView,
  FaviconsView,
  NoteOpenRequested,
  NoteChanges,
  ResourceChanged,
  DownloadsChanged,
  BlockerStatusChanged,
  BrowserCredentialCapabilityChanged,
  FocusChanged,
  ExtensionActionFailed,
  ExtensionActionShortcut,
  ExtensionActionsChanged,
  FindResultView,
  ImportJobView,
  ItemsChanged,
  KeymapChanged,
  LayoutChanged,
  MediaPlayerChanged,
  OperationProcessed,
  PagePermissionPromptChanged,
  RuntimeStatusChanged,
  SearchChanged,
  TabChanged,
  UiCommand,
  VoiceChanged,
  VoiceLevels,
  SwipeScroll,
  WebExtensionAccessRequested,
  WebExtensionStoreClicked,
} from "./bindings";

type PayloadEvent<T> = { payload: T };
type Listener<T> = (event: PayloadEvent<T>) => void;
type PresentationTab = { tab: TabChanged; active: string | null };

function scopedEvent<T>(name: string) {
  return {
    listen(listener: Listener<T>): Promise<() => void> {
      const handler = (event: Event) => listener({ payload: (event as CustomEvent<T>).detail });
      window.addEventListener(name, handler);
      return Promise.resolve(() => window.removeEventListener(name, handler));
    },
  };
}

// Rust delivers these directly into the intended WebView. Do not replace this
// with @tauri-apps/api/event: its listen command accepts a caller-selected
// target, which would let the launcher panel subscribe to main-window state.
export const nativeEventNames = {
  windowsReputationManaged: "hash:windows-reputation-managed",
  workEnvironmentChanged: "hash:work-environment-changed",
  workChanged: "hash:work-changed",
  workHumanChanged: "hash:work-human-changed",
  workDecisionPreferenceChanged: "hash:work-decision-preference-changed",
  workModelsChanged: "hash:work-models-changed",
  pageAssistChanged: "hash:page-assist-changed",
  accountChanged: "hash:account-changed",
  panelState: "hash:panel-state",
  favicons: "hash:favicons",
  noteOpenRequested: "hash:note-open-requested",
  resourceChanged: "hash:resource-changed",
  notesChanged: "hash:notes-changed",
  downloadsChanged: "hash:downloads-changed",
  resourceClose: "hash:resource-close",
  resourceCloseCancelled: "hash:resource-close-cancelled",
  browserCredentialCapabilityChanged: "hash:browser-credential-capability",
  itemsChanged: "hash:items",
  tabChanged: "hash:tab",
  extensionActionsChanged: "hash:extension-actions",
  extensionActionFailed: "hash:extension-action-failed",
  extensionActionShortcut: "hash:extension-action-shortcut",
  pagePermissionPromptChanged: "hash:page-permission-prompt",
  webExtensionAccessRequested: "hash:web-extension-access",
  webExtensionStoreClicked: "hash:web-extension-store-click",
  webExtensionDropped: "hash:web-extension-dropped",
  browserReturn: "hash:browser-return",
  presentationTab: "hash:presentation-tab",
  uiCommand: "hash:ui-command",
  keymapChanged: "hash:keymap-changed",
  importProgress: "hash:import-progress",
  find: "hash:find",
  searchChanged: "hash:search",
  layoutChanged: "hash:layout",
  mediaPlayerChanged: "hash:media-player",
  operationProcessed: "hash:operation-processed",
  runtimeStatusChanged: "hash:runtime-status",
  blockerStatusChanged: "hash:blocker-status",
  focusChanged: "hash:focus",
  voiceChanged: "hash:voice-changed",
  voiceLevels: "hash:voice-levels",
  swipeScroll: "hash:swipe",
  profilesChanged: "hash:profiles",
} as const;

export const events = {
  windowsReputationManaged: scopedEvent<boolean>(nativeEventNames.windowsReputationManaged),
  workEnvironmentChanged: scopedEvent<WorkEnvironmentChanged>(
    nativeEventNames.workEnvironmentChanged,
  ),
  workChanged: scopedEvent<WorkChanged>(nativeEventNames.workChanged),
  workHumanChanged: scopedEvent<WorkHumanChanged>(nativeEventNames.workHumanChanged),
  workDecisionPreferenceChanged: scopedEvent<WorkDecisionPreferenceChanged>(
    nativeEventNames.workDecisionPreferenceChanged,
  ),
  workModelsChanged: scopedEvent<WorkModelsChanged>(nativeEventNames.workModelsChanged),
  pageAssistChanged: scopedEvent<PageAssistChanged>(nativeEventNames.pageAssistChanged),
  accountChanged: scopedEvent<AccountChanged>(nativeEventNames.accountChanged),
  favicons: scopedEvent<FaviconsView>(nativeEventNames.favicons),
  noteOpenRequested: scopedEvent<NoteOpenRequested>(nativeEventNames.noteOpenRequested),
  resourceClose: scopedEvent<string>(nativeEventNames.resourceClose),
  resourceCloseCancelled: scopedEvent<string>(nativeEventNames.resourceCloseCancelled),
  resourceChanged: scopedEvent<ResourceChanged>(nativeEventNames.resourceChanged),
  notesChanged: scopedEvent<NoteChanges>(nativeEventNames.notesChanged),
  downloadsChanged: scopedEvent<DownloadsChanged>(nativeEventNames.downloadsChanged),
  panelState: scopedEvent<PanelState>(nativeEventNames.panelState),
  browserCredentialCapabilityChanged: scopedEvent<BrowserCredentialCapabilityChanged>(
    nativeEventNames.browserCredentialCapabilityChanged,
  ),
  itemsChanged: scopedEvent<ItemsChanged>(nativeEventNames.itemsChanged),
  tabChanged: scopedEvent<TabChanged>(nativeEventNames.tabChanged),
  extensionActionsChanged: scopedEvent<ExtensionActionsChanged>(
    nativeEventNames.extensionActionsChanged,
  ),
  extensionActionFailed: scopedEvent<ExtensionActionFailed>(nativeEventNames.extensionActionFailed),
  extensionActionShortcut: scopedEvent<ExtensionActionShortcut>(
    nativeEventNames.extensionActionShortcut,
  ),
  pagePermissionPromptChanged: scopedEvent<PagePermissionPromptChanged>(
    nativeEventNames.pagePermissionPromptChanged,
  ),
  webExtensionAccessRequested: scopedEvent<WebExtensionAccessRequested>(
    nativeEventNames.webExtensionAccessRequested,
  ),
  webExtensionStoreClicked: scopedEvent<WebExtensionStoreClicked>(
    nativeEventNames.webExtensionStoreClicked,
  ),
  /** The path of an extension package dropped on the browser. */
  webExtensionDropped: scopedEvent<string>(nativeEventNames.webExtensionDropped),
  browserReturn: scopedEvent<ItemsChanged>(nativeEventNames.browserReturn),
  presentationTab: scopedEvent<PresentationTab>(nativeEventNames.presentationTab),
  uiCommand: scopedEvent<UiCommand>(nativeEventNames.uiCommand),
  keymapChanged: scopedEvent<KeymapChanged>(nativeEventNames.keymapChanged),
  importProgress: scopedEvent<ImportJobView>(nativeEventNames.importProgress),
  find: scopedEvent<FindResultView>(nativeEventNames.find),
  searchChanged: scopedEvent<SearchChanged>(nativeEventNames.searchChanged),
  layoutChanged: scopedEvent<LayoutChanged>(nativeEventNames.layoutChanged),
  mediaPlayerChanged: scopedEvent<MediaPlayerChanged>(nativeEventNames.mediaPlayerChanged),
  operationProcessed: scopedEvent<OperationProcessed>(nativeEventNames.operationProcessed),
  runtimeStatusChanged: scopedEvent<RuntimeStatusChanged>(nativeEventNames.runtimeStatusChanged),
  blockerStatusChanged: scopedEvent<BlockerStatusChanged>(nativeEventNames.blockerStatusChanged),
  focusChanged: scopedEvent<FocusChanged>(nativeEventNames.focusChanged),
  voiceChanged: scopedEvent<VoiceChanged>(nativeEventNames.voiceChanged),
  voiceLevels: scopedEvent<VoiceLevels>(nativeEventNames.voiceLevels),
  swipeScroll: scopedEvent<SwipeScroll>(nativeEventNames.swipeScroll),
  profilesChanged: scopedEvent<ProfilesView>(nativeEventNames.profilesChanged),
};
