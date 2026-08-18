import { loadNative, type DownloadResult, type MediaInfo } from './native.js';

export type { DownloadResult, MediaInfo };

/** Native addon / package version. */
export function version(): string {
  return loadNative().version();
}

/** Resolve media metadata for a URL. */
export function info(url: string): MediaInfo {
  const raw = loadNative().info(url);
  return {
    url: raw.url,
    title: raw.title ?? null,
    contentType: raw.contentType ?? null,
    contentLength: raw.contentLength ?? null,
    filename: raw.filename ?? null,
    extractor: raw.extractor,
  };
}

/** Download a URL into `outputDir`. */
export function download(url: string, outputDir: string): DownloadResult {
  const raw = loadNative().download(url, outputDir);
  return {
    path: raw.path,
    bytesWritten: raw.bytesWritten,
    filename: raw.filename,
  };
}

/** Create an empty credential vault file. */
export function createVault(vaultPath: string, password: string): string {
  return loadNative().createVault(vaultPath, password);
}
