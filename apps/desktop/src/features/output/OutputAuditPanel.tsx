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
  HumanAuditRequestDraft,
  OutputIndex,
  OutputIndexEntry,
} from "../../types";
import { isBrowserPreviewRuntime } from "../project-files";
import type { OutputStatusState } from "./hooks/useOutputStatus";
import "./OutputAuditPanel.css";

type OutputAuditPanelProps = {
  onAuditRequested?: () => void;
  outputStatusState: OutputStatusState;
  projectRoot: string | null;
};

type AuditPanelSource = "idle" | "loading" | "tauri" | "preview" | "unavailable";

export function OutputAuditPanel({ onAuditRequested, outputStatusState, projectRoot }: OutputAuditPanelProps) {
  const [auditIndex, setAuditIndex] = useState<AuditIndex | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [outputIndex, setOutputIndex] = useState<OutputIndex | null>(null);
  const [reason, setReason] = useState("");
  const [refreshToken, setRefreshToken] = useState(0);
  const [report, setReport] = useState<HumanAuditReport | null>(null);
  const [selectedAuditId, setSelectedAuditId] = useState<string | null>(null);
  const [selectedDeliveryRunId, setSelectedDeliveryRunId] = useState<string | null>(null);
  const [source, setSource] = useState<AuditPanelSource>("idle");
  const [submitting, setSubmitting] = useState(false);

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

        if (latestAudit) {
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
  const trimmedReason = reason.trim();
  const previewOnly = source === "preview";
  const requestDisabled = previewOnly || !selectedDelivery || !trimmedReason || submitting;

  async function loadAuditReport(auditId: string) {
    if (!projectRoot || isBrowserPreviewRuntime()) {
      return;
    }
    setError(null);
    setSelectedAuditId(auditId);
    try {
      setReport(await invoke<HumanAuditReport>("load_audit_report", { projectRoot, auditId }));
    } catch (caughtError) {
      setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
    }
  }

  async function requestHumanAudit() {
    if (!projectRoot || !selectedDelivery || requestDisabled) {
      return;
    }

    if (previewOnly) {
      setError("浏览器预览不写 .agentflow/output/audit；请在 Tauri Desktop 中触发人工审计。");
      return;
    }

    setError(null);
    setSubmitting(true);
    try {
      const nextReport = await invoke<HumanAuditReport>("request_human_audit", {
        projectRoot,
        draft: buildHumanAuditRequestDraft(selectedDelivery, trimmedReason),
      });
      setReport(nextReport);
      setReason("");
      setSelectedAuditId(nextReport.audit.auditId);
      setRefreshToken((current) => current + 1);
      onAuditRequested?.();
    } catch (caughtError) {
      setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <section className="output-audit-panel" aria-label="人工审计">
      <header className="output-audit-header">
        <div>
          <p>交付输出</p>
          <h2>人工审计</h2>
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
        <section className="output-audit-request" aria-label="请求人工审计">
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

          <div className="output-audit-field">
            <label htmlFor="human-audit-reason">审计原因</label>
            <textarea
              disabled={previewOnly}
              id="human-audit-reason"
              onChange={(event) => setReason(event.target.value)}
              placeholder={previewOnly ? "浏览器预览不写 audit。" : "请输入审计原因"}
              rows={3}
              value={reason}
            />
          </div>

          <button className="output-audit-button" disabled={requestDisabled} onClick={() => void requestHumanAudit()} type="button">
            {submitting ? "正在请求审计" : "请求人工审计"}
          </button>

          <p className="output-audit-note">
            {previewOnly
              ? "浏览器预览不写 .agentflow/output/audit；请在 Tauri Desktop 中触发人工审计。"
              : selectedDelivery
                ? `将自动审计 ${selectedDelivery.runId} 的 spec / issue / execute-run / evidence / release-delivery。`
                : "暂无可审计交付材料。"}
          </p>
          {error ? <p className="output-audit-error">{error}</p> : null}
        </section>

        <section className="output-audit-report" aria-label="审计报告">
          <div className="output-audit-report-header">
            <div>
              <p>最新审计报告</p>
              <strong>{selectedAudit?.auditId ?? "未请求审计"}</strong>
            </div>
            <span>{selectedAudit ? auditStatusLabel(selectedAudit.status) : "未请求审计"}</span>
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
              {source === "loading" ? "正在加载审计状态。" : "还没有审计报告。选择 delivery 并填写 reason 后可请求人工审计。"}
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

function buildHumanAuditRequestDraft(delivery: OutputIndexEntry, reason: string): HumanAuditRequestDraft {
  return {
    reason,
    scope: {
      description: "Human requested audit for Build Agent delivery.",
      refs: [
        {
          kind: "spec",
          id: delivery.sourceSpecId,
          path: `.agentflow/input/specs/approved/${delivery.sourceSpecId}/`,
        },
        {
          kind: "issue",
          id: delivery.issueId,
          path: `.agentflow/input/issues/${delivery.issueId}.json`,
        },
        {
          kind: "execute-run",
          id: delivery.runId,
          path: `.agentflow/execute/runs/${delivery.runId}/`,
        },
        {
          kind: "evidence",
          id: delivery.runId,
          path: `.agentflow/output/evidence/${delivery.runId}.json`,
        },
        {
          kind: "release-delivery",
          id: delivery.runId,
          path: `.agentflow/output/release/${delivery.runId}/delivery.json`,
        },
      ],
    },
  };
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

function auditStatusLabel(status: string) {
  const labels: Record<string, string> = {
    passed: "通过",
    "passed-with-warnings": "通过，有警告",
    failed: "失败",
    cancelled: "已取消",
  };
  return labels[status] ?? status;
}
