import type { MediaInfo, StrategyPlugin } from '@doki-land/prometheus-plugin';
import genericHttp from '@doki-land/prometheus-strategy-generic-http';
import localFile from '@doki-land/prometheus-strategy-local-file';
import { TorchRuntime } from '@doki-land/prometheus-torch';

const bundled: StrategyPlugin[] = [localFile, genericHttp];

export function bundledStrategies(): StrategyPlugin[] {
    return bundled;
}

export async function inspectWithStrategies(url: string): Promise<MediaInfo | null> {
    const torch = new TorchRuntime();
    for (const plugin of bundled) {
        if (!plugin.matches(url)) continue;
        return await plugin.inspect(url, { torch });
    }
    return null;
}
