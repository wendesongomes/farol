import type { Group } from "../group";
import { messages as t } from "../i18n";

export function GroupHeader({ group }: { group: Group }) {
  const title = group.kind === "pinned" ? t.pinnedGroup : group.kind === "outside" ? t.outsideGroup : group.name;
  return (
    <div className="group-header">
      <span className="group-name">{title}</span>
      {group.branch && <span className="group-branch">{group.branch}</span>}
      {group.path && (
        <span className="group-path" title={group.path}>
          {group.path}
        </span>
      )}
    </div>
  );
}
