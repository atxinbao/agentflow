import { useEffect, useState, type FormEvent } from "react";
import type { GoalRecord } from "../../../types";
import { linesToList, listToLines } from "../model/goalTreeUtils";

export function GoalEditor({
  goal,
  onArchive,
  onSave,
  saving,
}: {
  goal: GoalRecord;
  onArchive: () => void;
  onSave: (goalId: string, patch: Record<string, unknown>) => void;
  saving: boolean;
}) {
  const [title, setTitle] = useState(goal.human.title);
  const [objective, setObjective] = useState(goal.human.objective);
  const [scope, setScope] = useState(listToLines(goal.human.scope));
  const [nonGoals, setNonGoals] = useState(listToLines(goal.human.nonGoals));
  const [successCriteria, setSuccessCriteria] = useState(listToLines(goal.human.successCriteria));
  const [validationGate, setValidationGate] = useState(listToLines(goal.human.validationGate));
  const [closureGate, setClosureGate] = useState(listToLines(goal.human.closureGate));

  useEffect(() => {
    setTitle(goal.human.title);
    setObjective(goal.human.objective);
    setScope(listToLines(goal.human.scope));
    setNonGoals(listToLines(goal.human.nonGoals));
    setSuccessCriteria(listToLines(goal.human.successCriteria));
    setValidationGate(listToLines(goal.human.validationGate));
    setClosureGate(listToLines(goal.human.closureGate));
  }, [goal]);

  function submit(event: FormEvent) {
    event.preventDefault();
    onSave(goal.id, {
      title,
      objective,
      scope: linesToList(scope),
      nonGoals: linesToList(nonGoals),
      successCriteria: linesToList(successCriteria),
      validationGate: linesToList(validationGate),
      closureGate: linesToList(closureGate),
    });
  }

  return (
    <form className="goal-tree-editor" onSubmit={submit}>
      <EditorHeader eyebrow="Goal" id={goal.id} onArchive={onArchive} saving={saving} status={goal.status} title={goal.human.title} />
      <TextField label="标题" value={title} onChange={setTitle} />
      <TextArea label="目标" value={objective} onChange={setObjective} />
      <TextArea label="范围" value={scope} onChange={setScope} />
      <TextArea label="非目标" value={nonGoals} onChange={setNonGoals} />
      <TextArea label="成功标准" value={successCriteria} onChange={setSuccessCriteria} />
      <TextArea label="验证门槛" value={validationGate} onChange={setValidationGate} />
      <TextArea label="完成判定" value={closureGate} onChange={setClosureGate} />
      <EditorActions saving={saving} />
    </form>
  );
}

export function EditorHeader({
  eyebrow,
  id,
  onArchive,
  saving,
  status,
  title,
}: {
  eyebrow: string;
  id: string;
  onArchive: () => void;
  saving: boolean;
  status: string;
  title: string;
}) {
  return (
    <header className="goal-tree-editor-header">
      <div>
        <span className="goal-tree-kicker">{eyebrow} / {id}</span>
        <h2>{title || "未命名"}</h2>
      </div>
      <div className="goal-tree-editor-actions">
        <span className="goal-tree-status-pill">{status}</span>
        <button disabled={saving} onClick={onArchive} type="button">
          归档
        </button>
      </div>
    </header>
  );
}

export function TextField({
  label,
  onChange,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  value: string;
}) {
  return (
    <label className="goal-tree-field">
      <span>{label}</span>
      <input onChange={(event) => onChange(event.target.value)} value={value} />
    </label>
  );
}

export function TextArea({
  label,
  onChange,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  value: string;
}) {
  return (
    <label className="goal-tree-field">
      <span>{label}</span>
      <textarea onChange={(event) => onChange(event.target.value)} rows={4} value={value} />
    </label>
  );
}

export function EditorActions({ saving }: { saving: boolean }) {
  return (
    <div className="goal-tree-form-actions">
      <button disabled={saving} type="submit">
        {saving ? "保存中..." : "保存合同"}
      </button>
    </div>
  );
}
