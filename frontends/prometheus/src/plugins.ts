import type { MediaInfo, Plugin, PluginContext } from '@doki-land/prometheus-plugin';
import genericHttp from '@doki-land/prometheus-plugin-generic-http';
import localFile from '@doki-land/prometheus-plugin-local-file';

const bundled: Plugin[] = [localFile, genericHttp];

export function bundledPlugins(): Plugin[] {
    return bundled;
}

/**
 * Optional JS plugin path. Does not load Torch — the product package must not
 * depend on `@doki-land/prometheus-torch`. Plugins that need script/WASM hosts
 * should check `ctx.torch` and tell the user to install that upgrade package.
 */
export async function inspectWithPlugins(url: string): Promise<MediaInfo | null> {
    const ctx: PluginContext = {};
    for (const plugin of bundled) {
        if (!plugin.matches(url)) continue;
        return await plugin.inspect(url, ctx);
    }
    return null;
}
