import { useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Icons } from "../../components/Icons";

interface PhraseGroup {
  id: string;
  name: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface GroupChipsProps {
  groups: PhraseGroup[];
  selectedGroupId: string | null;
  onSelectGroup: (id: string) => void;
  onAddGroup: () => void;
  onManageGroups: () => void;
  onAddPhrase: () => void;
}

export function GroupChips({
  groups,
  selectedGroupId,
  onSelectGroup,
  onAddGroup,
  onManageGroups,
  onAddPhrase,
}: GroupChipsProps) {
  const { t } = useTranslation();
  const groupsScrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = groupsScrollRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
        e.preventDefault();
        el.scrollLeft += e.deltaY;
      }
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, []);

  return (
    <div className="phrase-groups">
      <div className="groups-scroll" ref={groupsScrollRef}>
        {groups.map((g) => (
          <button
            key={g.id}
            className={`group-chip ${g.id === selectedGroupId ? "active" : ""}`}
            onClick={() => onSelectGroup(g.id)}
          >
            {g.name}
          </button>
        ))}
      </div>
      <button className="group-add-btn" onClick={onAddGroup}>
        {Icons.add}
      </button>
      <button className="group-add-btn" onClick={onManageGroups} title={t("phrases.manageGroups")}>
        {Icons.edit}
      </button>
      {selectedGroupId && (
        <button className="phrase-add-btn" onClick={onAddPhrase}>
          {Icons.add}
          <span>{t("phrases.newPhrase")}</span>
        </button>
      )}
    </div>
  );
}
