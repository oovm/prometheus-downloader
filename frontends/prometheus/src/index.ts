import { type DownloadResult, loadNative, type MediaInfo } from './native.js';
import { inspectWithStrategies } from './strategies.js';

export { bundledStrategies } from './strategies.js';
export type { DownloadResult, MediaInfo };

/** Native addon / package version. */
export function version(): string {
    return loadNative().version();
}

function fromNativeInfo(url: string): MediaInfo {
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

/** Resolve media metadata for a URL. Tries strategy plugins, then the native engine. */
export async function info(url: string): Promise<MediaInfo> {
    try {
        const fromPlugin = await inspectWithStrategies(url);
        if (fromPlugin) return fromPlugin;
    } catch {
        /* native fallback */
    }
    return fromNativeInfo(url);
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
