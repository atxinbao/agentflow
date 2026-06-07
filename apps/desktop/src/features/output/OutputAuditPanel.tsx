import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import {
  createBrowserPreviewAuditIndex,
  createBrowserPreviewHumanAuditReport,
  createBrowserPreviewOutputIndex,
} from "../../browserPreviewData";
import type {
  AuditIndex,
  AuditIndexEntry,
  HumanAuditReport,
  OutputIndex,
  OutputIndexEntry,
} from "../../types";
import { isBrowserPreviewRuntime } from "../project-files";
import type { OutputStatusState } from "./hooks/useOutputStatus";
import "./OutputAuditPanel.css";

type OutputAuditPanelProps = {
  outputStatusState: OutputStatusState;
  projectRoot: string | null;
};

type AuditPanelSource = "idle" | "loading" | "tauri" | "preview" | "unavailable";

export function OutputAuditPanel({ outputStatusState, projectRoot }: OutputAuditPanelProps) {
  const [auditIndex, setAuditIndex] = useState<AuditIndex | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [outputIndex, setOutputIndex] = useState<OutputIndex | null>(null);
  const [refreshToken, setRefreshToken] = useState(0);
  const [report, setReport] = useState<HumanAuditReport | null>(null);
  const [selectedAuditId, setSelectedAuditId] = useState<string | null>(null);
  const [selectedDeliveryRunId, setSelectedDeliveryRunId] = useState<string | null>(null);
  const [source, setSource] = useState<AuditPanelSource>("idle");

  useEffect(() => {
    if (!projectRoot) {
      setAuditIndex(null);
      setError(null);
      setOutputIndex(null);
      setReport(null);
      setSelectedAuditId(null);
      setSelectedDeliveryRunId(null);
      setSource("idle");
      return;
    }

    if (isBrowserPreviewRuntime()) {
      const previewOutputIndex = createBrowserPreviewOutputIndex();
      const previewAuditIndex = createBrowserPreviewAuditIndex();
      setAuditIndex(previewAuditIndex);
      setError(null);
      setOutputIndex(previewOutputIndex);
      setReport(createBrowserPreviewHumanAuditReport());
      setSelectedAuditId(previewAuditIndex.audits.at(-1)?.auditId ?? null);
      setSelectedDeliveryRunId(previewOutputIndex.releaseDeliveries.at(-1)?.runId ?? null);
      setSource("preview");
      return;
    }

    let cancelled = false;
    setError(null);
    setSource("loading");

    void Promise.all([
      invoke<OutputIndex>("load_output_index", { projectRoot }),
      invoke<AuditIndex>("load_audit_index", { projectRoot }),
    ])
      .then(async ([nextOutputIndex, nextAuditIndex]) => {
        if (cancelled) {
          return;
        }

        setOutputIndex(nextOutputIndex);
        setAuditIndex(nextAuditIndex);
        setSource("tauri");
        setSelectedDeliveryRunId((current) =>
          current && nextOutputIndex.releaseDeliveries.some((delivery) => delivery.runId === current)
            ? current
            : latestOutputEntry(nextOutputIndex.releaseDeliveries)?.runId ?? null,
        );

        const latestAudit = latestAuditEntry(nextAuditIndex.audits);
        setSelectedAuditId((current) =>
          current && nextAuditIndex.audits.some((audit) => audit.auditId === current)
            ? current
            : latestAudit?.auditId ?? null,
        );

        if (latestAudit && auditPanelHasReport(latestAudit)) {
          const nextReport = await invoke<HumanAuditReport>("load_audit_report", {
            projectRoot,
            auditId: latestAudit.auditId,
          });
          if (!cancelled) {
            setReport(nextReport);
          }
        } else {
          setReport(null);
        }
      })
      .catch((caughtError) => {
        if (!cancelled) {
          setAuditIndex(null);
          setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
          setOutputIndex(null);
          setReport(null);
          setSource("unavailable");
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot, refreshToken]);

  const releaseDeliveries = useMemo(
    () => [...(outputIndex?.releaseDeliveries ?? [])].sort(compareOutputEntries),
    [outputIndex?.releaseDeliveries],
  );
  const audits = useMemo(() => [...(auditIndex?.audits ?? [])].sort(compareAuditEntries), [auditIndex?.audits]);
  const selectedDelivery =
    releaseDeliveries.find((delivery) => delivery.runId === selectedDeliveryRunId) ?? releaseDeliveries.at(-1) ?? null;
  const selectedAudit = audits.find((audit) => audit.auditId === selectedAuditId) ?? audits.at(-1) ?? null;
  const selectedDeliveryAudit = selectedDelivery ? findPanelAuditForDelivery(audits, selectedDelivery.runId) : null;

  async function loadAuditReport(auditId: string) {
    if (!projectRoot || isBrowserPreviewRuntime()) {
      return;
    }
    setError(null);
    setSelectedAuditId(auditId);
    const selected = audits.find((audit) => audit.auditId === auditId) ?? null;
    if (!auditPanelHasReport(selected)) {
      setReport(null);
      return;
    }
    try {
      setReport(await invoke<HumanAuditReport>("load_audit_report", { projectRoot, auditId }));
    } catch (caughtError) {
      setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
    }
  }

  return (
    <section className="output-audit-panel" aria-label="审计状态">
      <header className="output-audit-header">
        <div>
          <p>交付输出</p>
          <h2>审计状态</h2>
        </div>
        <div className="output-audit-metrics" aria-label="交付输出统计">
          <Metric label="证据" value={outputStatusState.status?.summary.evidence ?? outputIndex?.evidence.length ?? 0} />
          <Metric label="交付" value={outputStatusState.status?.summary.releaseDeliveries ?? releaseDeliveries.length} />
          <Metric label="审计" value={outputStatusState.status?.summary.audits ?? audits.length} />
          <Metric
            label="未完成"
            value={
              outputStatusState.status
                ? outputStatusState.status.summary.incompleteEvidence + outputStatusState.status.summary.incompleteDeliveries
                : 0
            }
          />
        </div>
      </header>

      <div className="output-audit-body">
        <section className="output-audit-request" aria-label="审计规则">
          <div className="output-audit-field">
            <label htmlFor="human-audit-delivery">交付材料</label>
            <select
              disabled={releaseDeliveries.length === 0 || source === "loading"}
              id="human-audit-delivery"
              onChange={(event) => setSelectedDeliveryRunId(event.target.value)}
              value={selectedDeliveryRunId ?? ""}
            >
              {releaseDeliveries.length === 0 ? <option value="">暂无可审计交付材料</option> : null}
              {releaseDeliveries.map((delivery) => (
                <option key={delivery.runId} value={delivery.runId}>
                  {delivery.runId} · {delivery.issueId || "未记录 issue"} · {delivery.status}
                </option>
              ))}
            </select>
          </div>

          <p className="output-audit-note">
            {selectedDelivery
              ? selectedDeliveryAudit
                ? `${selectedDelivery.runId}：${auditTriggerLabel(selectedDeliveryAudit.trigger)}，${auditStatusLabel(selectedDeliveryAudit.status)}。`
                : `${selectedDelivery.runId}：交付已生成，暂无审计请求。任务完成不会自动触发审计。`
              : "暂无可审计交付材料。"}
          </p>
          <button className="output-audit-button" onClick={() => setRefreshToken((current) => current + 1)} type="button">
            刷新审计状态
          </button>
          {error ? <p className="output-audit-error">{error}</p> : null}
        </section>

        <section className="output-audit-report" aria-label="审计报告">
          <div className="output-audit-report-header">
            <div>
              <p>最新审计报告</p>
              <strong>{selectedAudit?.auditId ?? "未登记审计"}</strong>
            </div>
            <span>{selectedAudit ? auditStatusLabel(selectedAudit.status) : "未登记审计"}</span>
          </div>

          {audits.length > 0 ? (
            <div className="output-audit-list" aria-label="审计报告列表">
              {audits.map((audit) => (
                <button
                  className={audit.auditId === selectedAuditId ? "selected" : ""}
                  key={audit.auditId}
                  onClick={() => void loadAuditReport(audit.auditId)}
                  type="button"
                >
                  <span>{audit.auditId}</span>
                  <span>{auditTriggerLabel(audit.trigger)}</span>
                  <span>{auditStatusLabel(audit.status)}</span>
                </button>
              ))}
            </div>
          ) : null}

          {report ? (
            <div className="output-audit-report-content">
              <pre className="output-audit-markdown">{report.reportMarkdown}</pre>
              <JsonDetails label="Findings" value={report.findings} />
              <details>
                <summary>Checklist</summary>
                <pre>{report.checklistMarkdown}</pre>
              </details>
              <JsonDetails label="Evidence map" value={report.evidenceMap} />
              <JsonDetails label="Traceability" value={report.traceability} />
            </div>
          ) : (
            <p className="output-audit-empty">
              {source === "loading"
                ? "正在加载审计状态。"
                : selectedAudit?.status === "requested"
                  ? "审计请求已登记，等待 Agent 写入 audit report。"
                  : "还没有审计报告。App 只展示审计状态，不创建审计。"}
            </p>
          )}
        </section>
      </div>
    </section>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <span>
      <small>{label}</small>
      <strong>{value}</strong>
    </span>
  );
}

function JsonDetails({ label, value }: { label: string; value: unknown }) {
  return (
    <details>
      <summary>{label}</summary>
      <pre>{JSON.stringify(value, null, 2)}</pre>
    </details>
  );
}

function latestOutputEntry(entries: OutputIndexEntry[]) {
  return [...entries].sort(compareOutputEntries).at(-1) ?? null;
}

function latestAuditEntry(entries: AuditIndexEntry[]) {
  return [...entries].sort(compareAuditEntries).at(-1) ?? null;
}

function compareOutputEntries(left: OutputIndexEntry, right: OutputIndexEntry) {
  if (left.updatedAt !== right.updatedAt) {
    return left.updatedAt - right.updatedAt;
  }
  return left.runId.localeCompare(right.runId);
}

function compareAuditEntries(left: AuditIndexEntry, right: AuditIndexEntry) {
  if (left.requestedAt !== right.requestedAt) {
    return left.requestedAt - right.requestedAt;
  }
  return left.auditId.localeCompare(right.auditId);
}

function auditPanelHasReport(audit: AuditIndexEntry | null | undefined) {
  return Boolean(audit && audit.status !== "requested" && audit.status !== "running");
}

function findPanelAuditForDelivery(audits: AuditIndexEntry[], runId: string) {
  return (
    [...audits]
      .reverse()
      .find(
        (audit) =>
          audit.sourceRunId === runId ||
          audit.sourceDeliveryId === runId ||
          audit.auditId.includes(runId),
      ) ?? null
  );
}

function auditTriggerLabel(trigger: AuditIndexEntry["trigger"] | undefined) {
  switch (trigger) {
    case "release-auto":
      return "交付关联审计";
    case "human-via-agent":
      return "人类通过 Agent 触发";
    default:
      return "旧审计记录";
  }
}

function auditStatusLabel(status: string) {
  const labels: Record<string, string> = {
    requested: "等待 Agent 审计",
    running: "审计中",
    passed: "通过",
    "passed-with-warnings": "通过，有警告",
    failed: "失败",
    cancelled: "已取消",
  };
  return labels[status] ?? status;
}
