import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { usePhraseStore } from "../../stores/phraseStore";
import SearchInput from "../../components/SearchInput";
import { GroupChips } from "./GroupChips";
import { PhraseList } from "./PhraseList";
import { GroupDialog } from "./GroupDialog";
import { PhraseDialog } from "./PhraseDialog";
import { ManageGroupsDialog } from "./ManageGroupsDialog";

export default function PhrasePage() {
  const { t } = useTranslation();
  const [groupDialogOpen, setGroupDialogOpen] = useState(false);
  const [phraseDialogOpen, setPhraseDialogOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [groupName, setGroupName] = useState("");
  const [phraseRemark, setPhraseRemark] = useState("");
  const [phraseContent, setPhraseContent] = useState("");
  const [phraseError, setPhraseError] = useState(false);
  const [manageGroupsOpen, setManageGroupsOpen] = useState(false);
  const [renameId, setRenameId] = useState<string | null>(null);
  const [renameName, setRenameName] = useState("");

  const {
    groups,
    phrases,
    selectedGroupId,
    search,
    loading,
    setSearch,
    setSelectedGroup,
    init,
    loadPhrases,
    createGroup,
    updateGroup,
    createPhrase,
    updatePhrase,
    deletePhrase,
    deleteGroup,
    pastePhrase,
  } = usePhraseStore();

  useEffect(() => {
    init();
  }, []);

  useEffect(() => {
    if (selectedGroupId) {
      loadPhrases(selectedGroupId);
    }
  }, [selectedGroupId]);

  const openNewGroup = () => {
    setEditingId(null);
    setGroupName("");
    setGroupDialogOpen(true);
  };

  const handleSaveGroup = async () => {
    if (groupName.trim()) {
      if (editingId) {
        await updateGroup(editingId, groupName.trim());
      } else {
        await createGroup(groupName.trim());
      }
    }
    setGroupDialogOpen(false);
  };

  const openNewPhrase = () => {
    setEditingId(null);
    setPhraseRemark("");
    setPhraseContent("");
    setPhraseError(false);
    setPhraseDialogOpen(true);
  };

  const openEditPhrase = (p: { id: string; title: string; content: string }) => {
    setEditingId(p.id);
    setPhraseRemark(p.title);
    setPhraseContent(p.content);
    setPhraseError(false);
    setPhraseDialogOpen(true);
  };

  const handleSavePhrase = async () => {
    if (!phraseContent.trim()) {
      setPhraseError(true);
      return;
    }
    setPhraseError(false);
    if (editingId) {
      await updatePhrase(editingId, phraseRemark.trim(), phraseContent.trim());
    } else if (selectedGroupId) {
      await createPhrase(selectedGroupId, phraseRemark.trim(), phraseContent.trim());
    }
    setPhraseDialogOpen(false);
  };

  const openManageGroups = () => {
    setRenameId(null);
    setRenameName("");
    setManageGroupsOpen(true);
  };

  const startRename = (id: string, name: string) => {
    setRenameId(id);
    setRenameName(name);
  };

  const handleRename = async () => {
    if (renameId && renameName.trim()) {
      await updateGroup(renameId, renameName.trim());
    }
    setRenameId(null);
    setRenameName("");
  };

  const handleDeleteGroup = async (id: string) => {
    await deleteGroup(id);
    if (groups.length <= 1) {
      setManageGroupsOpen(false);
    }
  };

  return (
    <div className="phrase-page">
      <div className="page-search">
        <SearchInput
          placeholder={t("phrases.search")}
          value={search}
          onChange={setSearch}
        />
      </div>

      <GroupChips
        groups={groups}
        selectedGroupId={selectedGroupId}
        onSelectGroup={setSelectedGroup}
        onAddGroup={openNewGroup}
        onManageGroups={openManageGroups}
        onAddPhrase={openNewPhrase}
      />

      <PhraseList
        phrases={phrases}
        loading={loading}
        selectedGroupId={selectedGroupId}
        onPaste={pastePhrase}
        onEdit={openEditPhrase}
        onDelete={deletePhrase}
      />

      <GroupDialog
        open={groupDialogOpen}
        editingId={editingId}
        groupName={groupName}
        setGroupName={setGroupName}
        onSave={handleSaveGroup}
        onClose={() => setGroupDialogOpen(false)}
      />

      <PhraseDialog
        open={phraseDialogOpen}
        editingId={editingId}
        phraseRemark={phraseRemark}
        phraseContent={phraseContent}
        phraseError={phraseError}
        setPhraseRemark={setPhraseRemark}
        setPhraseContent={(content) => {
          setPhraseContent(content);
          if (content.trim()) setPhraseError(false);
        }}
        onSave={handleSavePhrase}
        onClose={() => setPhraseDialogOpen(false)}
      />

      <ManageGroupsDialog
        open={manageGroupsOpen}
        groups={groups}
        renameId={renameId}
        renameName={renameName}
        setRenameName={setRenameName}
        onStartRename={startRename}
        onRename={handleRename}
        onDeleteGroup={handleDeleteGroup}
        onClose={() => setManageGroupsOpen(false)}
      />
    </div>
  );
}
