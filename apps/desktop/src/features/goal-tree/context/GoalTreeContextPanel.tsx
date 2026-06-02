import { FileSearch, RefreshCw } from "lucide-react";
import type { GoalTreeIssueContextSnapshot, GoalTreeSnapshot, IssueRecord } from "../../../types";

export function GoalTreeContextPanel({
  context,
  contextLoading,
  issue,
  onOpenFile,
  onPrepareContext,
  snapshot,
}: {
  context: GoalTreeIssueContextSnapshot | null;
  contextLoading: boolean;
  issue: IssueRecord | null;
  onOpenFile: (relativePath: string) => void;
  onPrepareContext: (issueId: string) => void;
  snapshot: GoalTreeSnapshot;
}) {
  const warnings = snapshot.validation.warnings.filter((warning) => !issue || warning.objectId === issue.id);
  const recommendedFiles =
    context?.recommendedFiles.length || context?.recommendedTests.length
      ? [...(context?.recommendedFiles ?? []), ...(context?.recommendedTests ?? [])]
      : issue?.agentDraft.suggestedFiles.map((path) => ({ path, reason: "agent draft", score: 0 })) ?? [];

  return (
    <aside className="goal-tree-context-panel" aria-label="Goal Tree Context">
      <header>
        <span className="goal-tree-kicker">Context</span>
        <h2>代码现场</h2>
      </header>

      {issue ? (
        <button
          className="goal-tree-secondary-action"
          disabled={contextLoading}
          onClick={() => onPrepareContext(issue.id)}
          type="button"
        >
          <RefreshCw size={15} className={contextLoading ? "spin" : undefined} />
          准备 Graph Context
        </button>
      ) : (
        <p className="goal-tree-muted">选择 Issue 后可查看推荐文件。</p>
      )}

      <section>
        <h3>推荐文件</h3>
        {recommendedFiles.length === 0 ? (
          <p className="goal-tree-muted">暂无推荐文件。Graph 缺失时仍可编辑目标树。</p>
        ) : (
          <div className="goal-tree-file-list">
            {recommendedFiles.map((file) => (
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
