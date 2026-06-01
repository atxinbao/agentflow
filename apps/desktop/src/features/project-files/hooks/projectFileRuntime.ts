export function isBrowserPreviewRuntime() {
  if (typeof window === "undefined") {
    return false;
  }

  return !("__TAURI_INTERNALS__" in window);
}

export function readableProjectFilesError(error: unknown) {
  if (isBrowserPreviewRuntime()) {
    return "当前为浏览器预览，正在显示 Mock 项目文件；真实内容请在桌面客户端查看。";
  }
  return error instanceof Error ? error.message : String(error);
}
