import { FileSearch } from "lucide-react";
import type { GoalTreeSnapshot, IssueRecord } from "../../../types";

export function GoalTreeContextPanel({
  issue,
  onOpenFile,
  snapshot,
}: {
  issue: IssueRecord | null;
  onOpenFile: (relativePath: string) => void;
  snapshot: GoalTreeSnapshot;
}) {
  const warnings = snapshot.validation.warnings.filter((warning) => !issue || warning.objectId === issue.id);
  const recommendedFiles = issue?.agentDraft.suggestedFiles.map((path) => ({ path, reason: "Agent Draft", score: 0 })) ?? [];
  const recommendedTests = issue?.agentDraft.suggestedTests.map((path) => ({ path, reason: "Agent Draft", score: 0 })) ?? [];
  const readonlyRecommendations = [...recommendedFiles, ...recommendedTests];

  return (
    <aside className="goal-tree-context-panel" aria-label="Goal Tree Context">
      <header>
        <span className="goal-tree-kicker">Context</span>
        <h2>代码现场</h2>
      </header>

      {issue ? (
        <section>
          <h3>Context Pack</h3>
          <p className="goal-tree-muted">
            {issue.system.graphContextPackPath ?? "暂无 Agent 准备的 Panel Context。后续 Agent planning flow 会准备上下文。"}
          </p>
        </section>
      ) : (
        <p className="goal-tree-muted">选择 Issue 后可查看推荐文件。</p>
      )}

      <section>
        <h3>推荐文件</h3>
        {readonlyRecommendations.length === 0 ? (
          <p className="goal-tree-muted">暂无推荐文件。Panel 尚未 ready 时，Context 推荐可能不可用。</p>
        ) : (
          <div className="goal-tree-file-list">
            {readonlyRecommendations.map((file) => (
              <button key={`${file.path}-${file.reason}`} onClick={() => onOpenFile(file.path)} type="button">
                <FileSearch size={15} />
                <span>{file.path}</span>
                <small>{file.reason}</small>
              </button>
            ))}
          </div>
        )}
      </section>

      <section>
        <h3>完整性提示</h3>
        {[...snapshot.validation.errors, ...warnings].length === 0 ? (
          <p className="goal-tree-muted">当前目标树没有阻塞项。</p>
        ) : (
          <ul className="goal-tree-warning-list">
            {[...snapshot.validation.errors, ...warnings].slice(0, 8).map((item) => (
              <li key={`${item.code}-${item.objectId ?? "tree"}`}>
                <strong>{item.code}</strong>
                <span>{item.message}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </aside>
  );
}
