import { useState, type CSSProperties } from "react";
import { Button } from "./Button";

type CopyableCodeBlockProps = {
  content: string;
  language?: string;
  maxHeight?: number;
  title: string;
};

type CodeBlockStyle = CSSProperties & {
  "--af-code-max-height"?: string;
};

export function CopyableCodeBlock({ content, language = "text", maxHeight, title }: CopyableCodeBlockProps) {
  const [copied, setCopied] = useState(false);
  const style: CodeBlockStyle = maxHeight ? { "--af-code-max-height": `${maxHeight}px` } : {};

  async function copyContent() {
    try {
      await navigator.clipboard.writeText(content);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1400);
    } catch {
      setCopied(false);
    }
  }

  return (
    <section className="af-copyable-code-block" data-agentflow-component="copyable-code-block" style={style}>
      <header className="af-copyable-code-block-header">
        <div>
          <strong>{title}</strong>
          <span> · {language}</span>
        </div>
        <Button aria-label={`复制 ${title}`} onClick={() => void copyContent()} size="sm" variant="secondary">
          {copied ? "已复制" : "复制"}
        </Button>
      </header>
      <pre className="af-code-text">
        <code>{content}</code>
      </pre>
      <span aria-live="polite" hidden>
        {copied ? "复制成功" : ""}
      </span>
    </section>
  );
}
