import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const pkgRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');

export type MediaInfo = {
  url: string;
  title?: string | null;
  contentType?: string | null;
  contentLength?: number | null;
  filename?: string | null;
  extractor: string;
};

export type DownloadResult = {
  path: string;
  bytesWritten: number;
  filename: string;
};

type NativeAddon = {
  version: () => string;
  info: (url: string) => {
    url: string;
    title?: string | null;
    contentType?: string | null;
    contentLength?: number | null;
    filename?: string | null;
    extractor: string;
  };
  download: (
    url: string,
    outputDir: string,
  ) => {
    path: string;
    bytesWritten: number;
    filename: string;
  };
  createVault: (path: string, password: string) => string;
};

let cached: NativeAddon | undefined;

export function loadNative(): NativeAddon {
  if (cached) return cached;
  const candidates = [
    path.join(pkgRoot, 'prometheus.node'),
    path.join(pkgRoot, 'prometheus.win32-x64-msvc.node'),
  ];
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      cached = require(candidate) as NativeAddon;
      return cached;
    }
  }
  throw new Error(
    'prometheus native addon not found; run `pnpm napi:build` from the repository root',
  );
}
