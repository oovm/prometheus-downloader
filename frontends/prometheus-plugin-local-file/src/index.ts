import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type { MediaInfo, Plugin, PluginContext } from '@doki-land/prometheus-plugin';

const ID = 'local-file';

export function matches(url: string): boolean {
    return url.trim().toLowerCase().startsWith('file:');
}

export async function inspect(url: string, _ctx: PluginContext): Promise<MediaInfo> {
    const trimmed = url.trim();
    const filePath = fileURLToPath(trimmed);
    const stat = fs.statSync(filePath);
    if (!stat.isFile()) {
        throw new Error(`not a file: ${filePath}`);
    }
    const filename = path.basename(filePath);
    return {
        url: trimmed,
        title: filename,
        contentType: null,
        contentLength: stat.size,
        filename,
        extractor: ID,
    };
}

export const plugin: Plugin = { id: ID, matches, inspect };
export default plugin;
