import { type DownloadResult, type DownloadWithEvents, loadNative, type MediaInfo, type ProgressEvent, type TransferInfo } from './native.js';
import { inspectWithPlugins } from './plugins.js';

export { bundledPlugins } from './plugins.js';
export type { DownloadResult, DownloadWithEvents, MediaInfo, ProgressEvent, TransferInfo };

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

/** Resolve media metadata for a URL. Tries plugins, then the native engine. */
export async function info(url: string): Promise<MediaInfo> {
    try {
        const fromPlugin = await inspectWithPlugins(url);
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

/** List in-process transfer backends linked into the native addon. */
export function listTransfers(): TransferInfo[] {
    return loadNative().listTransfers();
}

/** Download a URL into `outputDir` and return collected progress events. */
export function downloadWithEvents(url: string, outputDir: string): DownloadWithEvents {
    const raw = loadNative().downloadWithEvents(url, outputDir);
    return {
        path: raw.path,
        bytesWritten: raw.bytesWritten,
        filename: raw.filename,
        events: raw.events,
    };
}

/** Create an empty credential vault file. */
export function createVault(vaultPath: string, password: string): string {
    return loadNative().createVault(vaultPath, password);
}
