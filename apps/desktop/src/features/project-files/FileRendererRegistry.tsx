import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { codeToHtml } from "shiki";
import { useEffect, useMemo, useRef, useState } from "react";
import type { ProjectFileContent, ProjectFileEntry, ProjectFileTextRange } from "../../types";
import { isProjectCodeLanguage } from "./projectFileUtils";

type FileRendererKind =
  | "markdown"
  | "json"
  | "code"
  | "large-text"
  | "plain-text"
  | "table"
  | "pdf"
  | "image"
  | "media"
  | "docx"
  | "binary"
  | "unsupported";

const LARGE_TEXT_THRESHOLD_BYTES = 220 * 1024;
const LARGE_TEXT_LINE_HEIGHT = 22;
const LARGE_TEXT_VIEWPORT_HEIGHT = 520;
const LARGE_TEXT_OVERSCAN_LINES = 8;

export function resolveProjectFileRenderer(content: ProjectFileContent, entry: ProjectFileEntry | null): FileRendererKind {
  const name = content.name || entry?.name || "";
  const extension = (content.extension ?? entry?.extension ?? fileExtension(name)).toLowerCase();
  const mimeType = content.mimeType ?? "";
  const language = content.language.toLowerCase();
  const sizeBytes = content.sizeBytes ?? entry?.sizeBytes ?? 0;

  if (content.binaryPreview) {
    if (["csv", "tsv", "xlsx"].includes(extension) && content.dataUrl) return "table";
    if (extension === "docx" && content.dataUrl) return "docx";
    if (mimeType.startsWith("image/") && content.dataUrl) return "image";
    if ((mimeType.startsWith("audio/") || mimeType.startsWith("video/")) && content.dataUrl) return "media";
    if (mimeType === "application/pdf" && content.dataUrl) return "pdf";
    return "binary";
  }

  if (extension === "pdf" || mimeType === "application/pdf") return "pdf";
  if (["png", "jpg", "jpeg", "gif", "webp", "svg"].includes(extension) || mimeType.startsWith("image/")) return "image";
  if (["mp3", "wav", "ogg", "mp4", "webm"].includes(extension) || mimeType.startsWith("audio/") || mimeType.startsWith("video/")) {
    return "media";
  }
  if (["csv", "tsv", "xlsx"].includes(extension)) return "table";
  if (extension === "docx") return "docx";
  if (language === "markdown" || ["md", "mdx", "markdown"].includes(extension)) return "markdown";
  if (language === "json" || ["json", "jsonc"].includes(extension)) return "json";
  if (content.truncated || sizeBytes > LARGE_TEXT_THRESHOLD_BYTES) return "large-text";
  if (isProjectCodeLanguage(language, name)) return "code";
  if (content.content !== null && content.content !== undefined) return "plain-text";
  return "unsupported";
}

export function ProjectFileBodyRenderer({
  content,
  entry,
  onLoadTextRange,
}: {
  content: ProjectFileContent;
  entry: ProjectFileEntry | null;
  onLoadTextRange?: (relativePath: string, startLine: number, lineCount: number) => Promise<ProjectFileTextRange>;
}) {
  const renderer = resolveProjectFileRenderer(content, entry);
  const textContent = content.content ?? "";

  if (content.unsupportedReason && renderer !== "binary" && renderer !== "large-text") {
    return <UnsupportedFallbackReader title={content.name} reason={content.unsupportedReason} />;
  }

  switch (renderer) {
    case "markdown":
      return <MarkdownReader content={textContent} />;
    case "json":
      return <JsonReader content={textContent} />;
    case "code":
      return <CodeReader content={textContent} language={content.language} name={content.name} />;
    case "large-text":
      return <LargeTextReader content={content} onLoadTextRange={onLoadTextRange} previewContent={textContent} reason={content.unsupportedReason} />;
    case "table":
      return <TableReader content={textContent} dataUrl={content.dataUrl} name={content.name} />;
    case "pdf":
      return <PdfReader content={content} />;
    case "image":
      return <ImageReader content={content} />;
    case "media":
      return <MediaReader content={content} />;
    case "docx":
      return <DocxReader content={content} />;
    case "binary":
      return <BinaryFallbackReader content={content} />;
    case "plain-text":
      return <PlainTextReader content={textContent} />;
    default:
      return <UnsupportedFallbackReader title={content.name} reason={content.unsupportedReason ?? "当前格式暂无专用阅读器。"} />;
  }
}

function MarkdownReader({ content }: { content: string }) {
  return (
    <div className="project-markdown-reader">
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{content || "文件为空。"}</ReactMarkdown>
    </div>
  );
}

function CodeReader({ content, language, name }: { content: string; language: string; name: string }) {
  const [highlightedHtml, setHighlightedHtml] = useState<string | null>(null);
  const normalizedLanguage = useMemo(() => shikiLanguage(language, name), [language, name]);

  useEffect(() => {
    let canceled = false;
    async function highlightCode() {
      try {
        const html = await codeToHtml(content || " ", {
          lang: normalizedLanguage,
          theme: "github-dark-default",
        });
        if (!canceled) {
          setHighlightedHtml(html);
        }
      } catch {
        if (!canceled) {
          setHighlightedHtml(null);
        }
      }
    }
    void highlightCode();
    return () => {
      canceled = true;
    };
  }, [content, normalizedLanguage]);

  if (highlightedHtml) {
    return <div className="project-code-reader shiki-reader" dangerouslySetInnerHTML={{ __html: highlightedHtml }} />;
  }

  return (
    <pre className="project-code-reader">
      <code>{content}</code>
    </pre>
  );
}

function JsonReader({ content }: { content: string }) {
  let formattedContent = content;
  try {
    formattedContent = JSON.stringify(JSON.parse(content), null, 2);
  } catch {
    formattedContent = content;
  }
  return <CodeReader content={formattedContent} language="json" name="data.json" />;
}

function LargeTextReader({
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
  const lines = useMemo(() => ((activeRange?.content ?? previewContent) || "暂无文本预览。").split("\n"), [activeRange?.content, previewContent]);
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

function PlainTextReader({ content }: { content: string }) {
  const paragraphs = content
    .split(/\n{2,}/)
    .map((paragraph) => paragraph.trim())
    .filter(Boolean);
  return (
    <div className="project-plain-reader">
      {paragraphs.length === 0 ? <p>文件为空。</p> : paragraphs.map((paragraph) => <p key={paragraph}>{paragraph}</p>)}
    </div>
  );
}

function TableReader({ content, dataUrl, name }: { content: string; dataUrl?: string | null; name: string }) {
  const extension = fileExtension(name).toLowerCase();
  if (extension === "xlsx") {
    return <XlsxReader dataUrl={dataUrl} name={name} />;
  }

  const rows = parseDelimitedTable(content, name.endsWith(".tsv") ? "\t" : ",");
  if (rows.length === 0) {
    return <UnsupportedFallbackReader title={name} reason="表格文件暂无可展示文本内容。" />;
  }
  return <TableGrid rows={rows} />;
}

function XlsxReader({ dataUrl, name }: { dataUrl?: string | null; name: string }) {
  const [rows, setRows] = useState<string[][] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let canceled = false;
    async function renderWorkbook() {
      try {
        const data = dataUrlToUint8Array(dataUrl ?? "");
        if (!data) {
          throw new Error("缺少 XLSX 二进制预览数据。");
        }
        const xlsx = await import("xlsx");
        const workbook = xlsx.read(data, { type: "array", sheetRows: 80 });
        const sheetName = workbook.SheetNames[0];
        const sheet = sheetName ? workbook.Sheets[sheetName] : null;
        if (!sheet) {
          throw new Error("XLSX 文件没有可展示的工作表。");
        }
        const parsedRows = xlsx.utils.sheet_to_json<Array<string | number | boolean | null>>(sheet, {
          blankrows: false,
          header: 1,
          raw: false,
        });
        if (!canceled) {
          setRows(parsedRows.map((row) => row.map((cell) => (cell === null || cell === undefined ? "" : String(cell)))));
          setError(null);
        }
      } catch (error) {
        if (!canceled) {
          setRows(null);
          setError(error instanceof Error ? error.message : String(error));
        }
      }
    }
    void renderWorkbook();
    return () => {
      canceled = true;
    };
  }, [dataUrl]);

  if (rows) {
    return <TableGrid note="XLSX 已用 SheetJS 读取首个工作表前 80 行。" rows={rows} />;
  }
  return <UnsupportedFallbackReader title={name} reason={error ?? "正在读取 XLSX 表格预览。"} />;
}

function TableGrid({ rows, note }: { rows: string[][]; note?: string }) {
  return (
    <div className="project-table-reader">
      {note ? <p>{note}</p> : null}
      <table>
        <tbody>
          {rows.slice(0, 80).map((row, rowIndex) => (
            <tr key={`${rowIndex}-${row.join("|")}`}>
              {row.slice(0, 16).map((cell, cellIndex) => (
                <td key={`${cellIndex}-${cell}`}>{cell}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function PdfReader({ content }: { content: ProjectFileContent }) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [pageCount, setPageCount] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let canceled = false;
    async function renderPdf() {
      try {
        const canvas = canvasRef.current;
        const data = dataUrlToUint8Array(content.dataUrl ?? "");
        if (!canvas || !data) {
          throw new Error("缺少 PDF 二进制预览数据。");
        }

        const pdfjs = await import("pdfjs-dist");
        pdfjs.GlobalWorkerOptions.workerSrc = new URL("pdfjs-dist/build/pdf.worker.mjs", import.meta.url).toString();
        const pdf = await pdfjs.getDocument({ data }).promise;
        const page = await pdf.getPage(1);
        const baseViewport = page.getViewport({ scale: 1 });
        const scale = Math.min(1.4, 860 / baseViewport.width);
        const viewport = page.getViewport({ scale });
        const context = canvas.getContext("2d");
        if (!context) {
          throw new Error("无法创建 PDF canvas context。");
        }
        canvas.width = Math.floor(viewport.width);
        canvas.height = Math.floor(viewport.height);
        await page.render({ canvas, canvasContext: context, viewport }).promise;
        if (!canceled) {
          setPageCount(pdf.numPages);
          setError(null);
        }
      } catch (error) {
        if (!canceled) {
          setPageCount(null);
          setError(error instanceof Error ? error.message : String(error));
        }
      }
    }
    void renderPdf();
    return () => {
      canceled = true;
    };
  }, [content.dataUrl]);

  if (!content.dataUrl) {
    return <UnsupportedFallbackReader title={content.name} reason="PDF 文件已识别；当前文件过大或缺少本地预览数据，显示 metadata fallback。" />;
  }

  return (
    <div className="project-pdf-reader">
      <p>{error ?? (pageCount ? `PDF.js 已渲染第 1 页，共 ${pageCount} 页。` : "正在读取 PDF 预览。")}</p>
      <canvas ref={canvasRef} aria-label={`${content.name} PDF 第一页预览`} />
    </div>
  );
}

function ImageReader({ content }: { content: ProjectFileContent }) {
  if (content.dataUrl) {
    return (
      <div className="project-media-reader">
        <img alt={content.name} src={content.dataUrl} />
      </div>
    );
  }
  return <UnsupportedFallbackReader title={content.name} reason="图片文件已识别；当前缺少本地预览数据，显示 metadata fallback。" />;
}

function MediaReader({ content }: { content: ProjectFileContent }) {
  if (!content.dataUrl) {
    return <UnsupportedFallbackReader title={content.name} reason="媒体文件已识别；当前缺少本地预览数据，显示 metadata fallback。" />;
  }
  if (content.mimeType?.startsWith("video/")) {
    return (
      <div className="project-media-reader">
        <video controls src={content.dataUrl} />
      </div>
    );
  }
  return (
    <div className="project-media-reader">
      <audio controls src={content.dataUrl} />
    </div>
  );
}

function DocxReader({ content }: { content: ProjectFileContent }) {
  const [html, setHtml] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let canceled = false;
    async function renderDocx() {
      try {
        const arrayBuffer = dataUrlToArrayBuffer(content.dataUrl ?? "");
        if (!arrayBuffer) {
          throw new Error("缺少 DOCX 二进制预览数据。");
        }
        const mammoth = await import("mammoth");
        const result = await mammoth.convertToHtml({ arrayBuffer });
        if (!canceled) {
          setHtml(result.value || "<p>DOCX 文件为空。</p>");
          setError(result.messages.length > 0 ? result.messages.map((message) => message.message).join("；") : null);
        }
      } catch (error) {
        if (!canceled) {
          setHtml(null);
          setError(error instanceof Error ? error.message : String(error));
        }
      }
    }
    void renderDocx();
    return () => {
      canceled = true;
    };
  }, [content.dataUrl]);

  if (html) {
    return (
      <div className="project-docx-reader">
        {error ? <p>{error}</p> : null}
        <div dangerouslySetInnerHTML={{ __html: html }} />
      </div>
    );
  }

  return <UnsupportedFallbackReader title={content.name} reason={error ?? "正在用 mammoth.js 读取 DOCX 预览。"} />;
}

function BinaryFallbackReader({ content }: { content: ProjectFileContent }) {
  return (
    <div className="project-binary-reader">
      <p>{content.unsupportedReason ?? "未知二进制文件，显示十六进制预览。"}</p>
      <pre>
        <code>{content.binaryPreview || "暂无十六进制预览。"}</code>
      </pre>
    </div>
  );
}

function UnsupportedFallbackReader({ title, reason }: { title: string; reason: string }) {
  return (
    <section className="project-fallback-reader">
      <h3>{title}</h3>
      <p>{reason}</p>
      <p>文件不会空白：当前显示 metadata / fallback 状态，不执行命令，不写入工作区。</p>
    </section>
  );
}

function shikiLanguage(language: string, name: string) {
  const extension = fileExtension(name).toLowerCase();
  if (language === "config" || name === ".gitignore") return "ignore";
  if (language === "shell") return "bash";
  if (
    [
      "toml",
      "json",
      "yaml",
      "rust",
      "typescript",
      "javascript",
      "css",
      "html",
      "markdown",
      "python",
      "go",
      "java",
      "kotlin",
      "swift",
      "dart",
      "c",
      "cpp",
      "csharp",
      "php",
      "ruby",
      "sql",
      "powershell",
      "xml",
    ].includes(language)
  ) {
    return language;
  }
  if (language === "objective-c") return "objective-c";
  if (language === "dockerfile" || name.toLowerCase() === "dockerfile") return "dockerfile";
  if (language === "makefile" || name.toLowerCase() === "makefile") return "makefile";
  if (extension === "tsx") return "tsx";
  if (extension === "ts") return "typescript";
  if (extension === "rs") return "rust";
  if (extension === "py") return "python";
  if (extension === "go") return "go";
  if (extension === "kt") return "kotlin";
  if (extension === "cs") return "csharp";
  if (extension === "ps1") return "powershell";
  if (extension === "sh") return "bash";
  return "text";
}

function parseDelimitedTable(content: string, delimiter: string) {
  return content
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => line.split(delimiter).map((cell) => cell.trim()));
}

function dataUrlToUint8Array(dataUrl: string) {
  const arrayBuffer = dataUrlToArrayBuffer(dataUrl);
  return arrayBuffer ? new Uint8Array(arrayBuffer) : null;
}

function dataUrlToArrayBuffer(dataUrl: string) {
  const base64 = dataUrl.split(",", 2)[1];
  if (!base64) {
    return null;
  }
  const binary = window.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes.buffer;
}

function fileExtension(name: string) {
  const dotIndex = name.lastIndexOf(".");
  if (dotIndex <= 0 || dotIndex === name.length - 1) {
    return "";
  }
  return name.slice(dotIndex + 1);
}
