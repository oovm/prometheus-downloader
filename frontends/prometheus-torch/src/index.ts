import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import vm from 'node:vm';

const require = createRequire(import.meta.url);

export type TorchEvaluateOptions = {
    filename?: string;
    timeoutMs?: number;
    sandbox?: Record<string, unknown>;
};

/**
 * Isolated Node context for platform JavaScript and WebAssembly.
 * Not a security sandbox: do not treat it as isolation against hostile code.
 */
export class TorchRuntime {
    evaluateJavascript(source: string, options: TorchEvaluateOptions = {}): unknown {
        if (typeof source !== 'string' || source.length === 0) {
            throw new TypeError('evaluateJavascript requires a non-empty source string');
        }
        const sandbox: Record<string, unknown> = { ...(options.sandbox ?? {}) };
        const context = vm.createContext(sandbox);
        const script = new vm.Script(source, {
            filename: options.filename ?? 'torch.js',
        });
        return script.runInContext(context, {
            timeout: options.timeoutMs ?? 5_000,
        });
    }

    instantiateWasm(bytes: BufferSource, imports?: WebAssembly.Imports): Promise<WebAssembly.WebAssemblyInstantiatedSource> {
        return WebAssembly.instantiate(bytes, imports);
    }
}

/** Package version. */
export function version(): string {
    const pkgPath = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'package.json');
    const pkg = require(pkgPath) as { version: string };
    return pkg.version;
}
