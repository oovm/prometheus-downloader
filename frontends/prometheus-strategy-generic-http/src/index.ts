import type { MediaInfo, StrategyContext, StrategyPlugin } from '@doki-land/prometheus-plugin';

const ID = 'generic-http';

function filenameFromUrl(url: string): string | null {
    try {
        const { pathname } = new URL(url);
        const base = pathname.split('/').filter(Boolean).at(-1);
        if (!base) return null;
        return decodeURIComponent(base);
    } catch {
        return null;
    }
}

function filenameFromDisposition(header: string | null): string | null {
    if (!header) return null;
    const star = /filename\*\s*=\s*UTF-8''([^;]+)/i.exec(header);
    if (star?.[1]) return decodeURIComponent(star[1].trim().replace(/^"|"$/g, ''));
    const plain = /filename\s*=\s*"?([^";]+)"?/i.exec(header);
    if (plain?.[1]) return plain[1].trim();
    return null;
}

export async function inspect(url: string, _ctx: StrategyContext): Promise<MediaInfo> {
    const trimmed = url.trim();
    let contentType: string | null = null;
    let contentLength: number | null = null;
    let filename = filenameFromUrl(trimmed);

    const res = await fetch(trimmed, { method: 'HEAD', redirect: 'follow' });
    if (res.ok) {
        contentType = res.headers.get('content-type');
        const len = res.headers.get('content-length');
        if (len) {
            const n = Number(len);
            if (Number.isFinite(n)) contentLength = n;
        }
        filename = filenameFromDisposition(res.headers.get('content-disposition')) ?? filename;
    }

    return {
        url: trimmed,
        title: filename,
        contentType,
        contentLength,
        filename,
        extractor: ID,
    };
}

export function matches(url: string): boolean {
    const lower = url.trim().toLowerCase();
    return lower.startsWith('http://') || lower.startsWith('https://');
}

export const plugin: StrategyPlugin = {
    id: ID,
    matches,
    inspect,
};

export default plugin;
