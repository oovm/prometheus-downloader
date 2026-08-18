import type { MediaInfo, Plugin } from '@doki-land/prometheus-plugin';
import genericHttp from '@doki-land/prometheus-plugin-generic-http';
import localFile from '@doki-land/prometheus-plugin-local-file';
import { TorchRuntime } from '@doki-land/prometheus-torch';

const bundled: Plugin[] = [localFile, genericHttp];

export function bundledPlugins(): Plugin[] {
    return bundled;
}

export async function inspectWithPlugins(url: string): Promise<MediaInfo | null> {
    const torch = new TorchRuntime();
    for (const plugin of bundled) {
        if (!plugin.matches(url)) continue;
        return await plugin.inspect(url, { torch });
    }
    return null;
}
