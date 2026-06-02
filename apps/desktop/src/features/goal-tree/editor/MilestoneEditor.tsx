import { useEffect, useState, type FormEvent } from "react";
import type { MilestoneRecord } from "../../../types";
import { EditorActions, EditorHeader, TextArea, TextField } from "./GoalEditor";
import { linesToList, listToLines } from "../model/goalTreeUtils";

export function MilestoneEditor({
  milestone,
  onArchive,
  onSave,
  saving,
}: {
  milestone: MilestoneRecord;
  onArchive: () => void;
  onSave: (milestoneId: string, patch: Record<string, unknown>) => void;
  saving: boolean;
}) {
  const [title, setTitle] = useState(milestone.human.title);
  const [stageGoal, setStageGoal] = useState(milestone.human.stageGoal);
  const [entryCriteria, setEntryCriteria] = useState(listToLines(milestone.human.entryCriteria));
  const [scope, setScope] = useState(listToLines(milestone.human.scope));
  const [nonGoals, setNonGoals] = useState(listToLines(milestone.human.nonGoals));
  const [exitCriteria, setExitCriteria] = useState(listToLines(milestone.human.exitCriteria));
  const [nextGate, setNextGate] = useState(listToLines(milestone.human.nextGate));

  useEffect(() => {
    setTitle(milestone.human.title);
    setStageGoal(milestone.human.stageGoal);
    setEntryCriteria(listToLines(milestone.human.entryCriteria));
    setScope(listToLines(milestone.human.scope));
    setNonGoals(listToLines(milestone.human.nonGoals));
    setExitCriteria(listToLines(milestone.human.exitCriteria));
    setNextGate(listToLines(milestone.human.nextGate));
  }, [milestone]);

  function submit(event: FormEvent) {
    event.preventDefault();
    onSave(milestone.id, {
      title,
      stageGoal,
      entryCriteria: linesToList(entryCriteria),
      scope: linesToList(scope),
      nonGoals: linesToList(nonGoals),
      exitCriteria: linesToList(exitCriteria),
      nextGate: linesToList(nextGate),
    });
  }

  return (
    <form className="goal-tree-editor" onSubmit={submit}>
      <EditorHeader
        eyebrow="Milestone"
        id={milestone.id}
        onArchive={onArchive}
        saving={saving}
        status={milestone.status}
        title={milestone.human.title}
      />
      <TextField label="标题" value={title} onChange={setTitle} />
      <TextArea label="阶段目标" value={stageGoal} onChange={setStageGoal} />
      <TextArea label="进入标准" value={entryCriteria} onChange={setEntryCriteria} />
      <TextArea label="范围" value={scope} onChange={setScope} />
      <TextArea label="非目标" value={nonGoals} onChange={setNonGoals} />
      <TextArea label="退出标准" value={exitCriteria} onChange={setExitCriteria} />
      <TextArea label="下一阶段门" value={nextGate} onChange={setNextGate} />
      <EditorActions saving={saving} />
    </form>
  );
}
