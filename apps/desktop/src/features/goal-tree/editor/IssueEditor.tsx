import { useEffect, useState, type FormEvent } from "react";
import type { IssueRecord } from "../../../types";
import { EditorActions, EditorHeader, TextArea, TextField } from "./GoalEditor";
import { linesToList, listToLines } from "../model/goalTreeUtils";

export function IssueEditor({
  issue,
  onArchive,
  onSave,
  saving,
}: {
  issue: IssueRecord;
  onArchive: () => void;
  onSave: (issueId: string, patch: Record<string, unknown>) => void;
  saving: boolean;
}) {
  const [title, setTitle] = useState(issue.human.title);
  const [goal, setGoal] = useState(issue.human.goal);
  const [scope, setScope] = useState(listToLines(issue.human.scope));
  const [nonGoals, setNonGoals] = useState(listToLines(issue.human.nonGoals));
  const [dependencies, setDependencies] = useState(listToLines(issue.human.dependencies));
  const [acceptanceCriteria, setAcceptanceCriteria] = useState(listToLines(issue.human.acceptanceCriteria));
  const [validationCommands, setValidationCommands] = useState(listToLines(issue.human.validationCommands));
  const [evidenceRequirements, setEvidenceRequirements] = useState(listToLines(issue.human.evidenceRequirements));
  const [boundary, setBoundary] = useState(listToLines(issue.human.boundary));

  useEffect(() => {
    setTitle(issue.human.title);
    setGoal(issue.human.goal);
    setScope(listToLines(issue.human.scope));
    setNonGoals(listToLines(issue.human.nonGoals));
    setDependencies(listToLines(issue.human.dependencies));
    setAcceptanceCriteria(listToLines(issue.human.acceptanceCriteria));
    setValidationCommands(listToLines(issue.human.validationCommands));
    setEvidenceRequirements(listToLines(issue.human.evidenceRequirements));
    setBoundary(listToLines(issue.human.boundary));
  }, [issue]);

  function submit(event: FormEvent) {
    event.preventDefault();
    onSave(issue.id, {
      title,
      goal,
      scope: linesToList(scope),
      nonGoals: linesToList(nonGoals),
      dependencies: linesToList(dependencies),
      acceptanceCriteria: linesToList(acceptanceCriteria),
      validationCommands: linesToList(validationCommands),
      evidenceRequirements: linesToList(evidenceRequirements),
      boundary: linesToList(boundary),
    });
  }

  return (
    <form className="goal-tree-editor" onSubmit={submit}>
      <EditorHeader
        eyebrow="Issue"
        id={issue.id}
        onArchive={onArchive}
        saving={saving}
        status={issue.status}
        title={issue.human.title}
      />
      <TextField label="标题" value={title} onChange={setTitle} />
      <TextArea label="目标" value={goal} onChange={setGoal} />
      <TextArea label="范围" value={scope} onChange={setScope} />
      <TextArea label="非目标" value={nonGoals} onChange={setNonGoals} />
      <TextArea label="依赖" value={dependencies} onChange={setDependencies} />
      <TextArea label="验收标准" value={acceptanceCriteria} onChange={setAcceptanceCriteria} />
      <TextArea label="验证命令（V1 只记录，不执行）" value={validationCommands} onChange={setValidationCommands} />
      <TextArea label="证据要求" value={evidenceRequirements} onChange={setEvidenceRequirements} />
      <TextArea label="边界" value={boundary} onChange={setBoundary} />
      <EditorActions saving={saving} />
    </form>
  );
}
