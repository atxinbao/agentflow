import { useEffect, useMemo, useRef, useState } from "react";
import type { ProjectFileContent, ProjectFileTextRange } from "../../../../types";

const LARGE_TEXT_LINE_HEIGHT = 22;
const LARGE_TEXT_VIEWPORT_HEIGHT = 520;
const LARGE_TEXT_OVERSCAN_LINES = 8;

export const LARGE_TEXT_THRESHOLD_BYTES = 220 * 1024;

export function LargeTextReader({
  content,
  onLoadTextRange,
  previewContent,
  reason,
}: {
  content: ProjectFileContent;
  onLoadTextRange?: (relativePath: string, startLine: number, lineCount: number) => Promise<ProjectFileTextRange>;
  previewContent: string;
  reason?: string | null;
}) {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [activeRange, setActiveRange] = useState<ProjectFileTextRange | null>(null);
  const [targetLineInput, setTargetLineInput] = useState("1");
  const [rangeLoading, setRangeLoading] = useState(false);
  const [rangeError, setRangeError] = useState<string | null>(null);
  const rangeLineCount = Math.max(240, Math.ceil(LARGE_TEXT_VIEWPORT_HEIGHT / LARGE_TEXT_LINE_HEIGHT) * 8);
  const currentRangeStartLine = activeRange?.startLine ?? 1;
  const currentRangeEndLine = activeRange?.endLine ?? 0;
  const totalLineLabel = activeRange ? `共 ${activeRange.totalLines} 行` : "总行数待加载";
  const canRequestRange = Boolean(onLoadTextRange);
  const lines = useMemo(
    () => ((activeRange?.content ?? previewContent) || "暂无文本预览。").split("\n"),
    [activeRange?.content, previewContent],
  );
  const visibleLineCount = Math.ceil(LARGE_TEXT_VIEWPORT_HEIGHT / LARGE_TEXT_LINE_HEIGHT);
  const startLine = Math.max(0, Math.floor(scrollTop / LARGE_TEXT_LINE_HEIGHT) - LARGE_TEXT_OVERSCAN_LINES);
  const endLine = Math.min(lines.length, startLine + visibleLineCount + LARGE_TEXT_OVERSCAN_LINES * 2);
  const visibleLines = lines.slice(startLine, endLine);
  const totalHeight = Math.max(lines.length * LARGE_TEXT_LINE_HEIGHT, LARGE_TEXT_VIEWPORT_HEIGHT);

  useEffect(() => {
    setActiveRange(null);
    setTargetLineInput("1");
    setRangeError(null);
    setScrollTop(0);
    if (viewportRef.current) {
      viewportRef.current.scrollTop = 0;
    }
  }, [content.relativePath, previewContent]);

  async function loadRange(startLine: number) {
    if (!onLoadTextRange) {
      return;
    }

    const requestedStartLine = Math.max(1, Math.floor(startLine || 1));
    setRangeLoading(true);
    setRangeError(null);
    try {
      const nextRange = await onLoadTextRange(content.relativePath, requestedStartLine, rangeLineCount);
      setActiveRange(nextRange);
      setTargetLineInput(String(nextRange.startLine));
      setScrollTop(0);
      if (viewportRef.current) {
        viewportRef.current.scrollTop = 0;
      }
    } catch (error) {
      setRangeError(error instanceof Error ? error.message : String(error));
    } finally {
      setRangeLoading(false);
    }
  }

  function loadPreviousRange() {
    const currentStart = activeRange?.startLine ?? 1;
    void loadRange(Math.max(1, currentStart - rangeLineCount));
  }

  function loadNextRange() {
    const nextStart = activeRange?.endLine ? activeRange.endLine + 1 : Math.max(1, lines.length + 1);
    void loadRange(nextStart);
  }

  function jumpToTargetLine() {
    const targetLine = Number.parseInt(targetLineInput, 10);
    void loadRange(Number.isFinite(targetLine) ? targetLine : 1);
  }

  return (
    <div className="project-large-text-reader">
      {reason ? <p>{reason}</p> : null}
      <div className="project-large-text-toolbar" aria-label="大文本按行读取控制">
        <div>
          <strong>大文本只读窗口</strong>
          <span>
            当前段：{activeRange ? `${currentRangeStartLine}-${currentRangeEndLine}` : `1-${Math.max(lines.length, 1)} 预览`} / {totalLineLabel}
          </span>
        </div>
        <div className="project-large-text-actions">
          <button disabled={!canRequestRange || rangeLoading || currentRangeStartLine <= 1} onClick={loadPreviousRange} type="button">
            上一段
          </button>
          <button
            disabled={!canRequestRange || rangeLoading || (activeRange ? !activeRange.truncated : !content.truncated)}
            onClick={loadNextRange}
            type="button"
          >
            下一段
          </button>
          <label>
            跳到行号
            <input
              inputMode="numeric"
              min={1}
              onChange={(event) => setTargetLineInput(event.target.value)}
              type="number"
              value={targetLineInput}
            />
          </label>
          <button disabled={!canRequestRange || rangeLoading} onClick={jumpToTargetLine} type="button">
            跳转
          </button>
        </div>
      </div>
      {rangeError ? <p className="project-large-text-error">{rangeError}</p> : null}
      <div
        ref={viewportRef}
        className="project-large-text-viewport"
        onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
        role="region"
        aria-label="大文本虚拟滚动预览"
      >
        <div className="project-large-text-spacer" style={{ height: totalHeight }}>
          <pre className="project-large-text-lines" style={{ transform: `translateY(${startLine * LARGE_TEXT_LINE_HEIGHT}px)` }}>
            <code>
              {visibleLines
                .map((line, index) => `${String(currentRangeStartLine + startLine + index).padStart(6, " ")}  ${line}`)
                .join("\n")}
            </code>
          </pre>
        </div>
      </div>
    </div>
  );
}
