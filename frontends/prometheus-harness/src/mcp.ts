import { callTool, TOOL_DEFS } from './tools.js';

type JsonRpc = {
    jsonrpc?: string;
    id?: number | string | null;
    method?: string;
    params?: Record<string, unknown>;
};

function writeMessage(obj: unknown): void {
    const json = JSON.stringify(obj);
    const payload = Buffer.from(json, 'utf8');
    process.stdout.write(`Content-Length: ${payload.length}\r\n\r\n`);
    process.stdout.write(payload);
}

function reply(id: number | string | null | undefined, result: unknown): void {
    writeMessage({ jsonrpc: '2.0', id: id ?? null, result });
}

function fail(id: number | string | null | undefined, message: string): void {
    writeMessage({ jsonrpc: '2.0', id: id ?? null, error: { code: -32000, message } });
}

async function handle(msg: JsonRpc): Promise<void> {
    const method = msg.method ?? '';
    if (method === 'initialize') {
        reply(msg.id, {
            protocolVersion: '2024-11-05',
            capabilities: { tools: {} },
            serverInfo: { name: 'prometheus-harness', version: '0.0.1' },
        });
        return;
    }
    if (method === 'notifications/initialized' || method === 'initialized') {
        return;
    }
    if (method === 'ping') {
        reply(msg.id, {});
        return;
    }
    if (method === 'tools/list') {
        reply(msg.id, { tools: TOOL_DEFS });
        return;
    }
    if (method === 'tools/call') {
        const name = String(msg.params?.name ?? '');
        const args = (msg.params?.arguments as Record<string, unknown> | undefined) ?? {};
        try {
            const result = await callTool(name, args);
            reply(msg.id, { content: [{ type: 'text', text: JSON.stringify(result, null, 2) }] });
        } catch (err) {
            fail(msg.id, err instanceof Error ? err.message : String(err));
        }
        return;
    }
    if (msg.id !== undefined) {
        fail(msg.id, `unsupported method: ${method}`);
    }
}

export async function runMcpStdio(): Promise<void> {
    let buffer = Buffer.alloc(0);
    process.stdin.on('data', (chunk: Buffer | string) => {
        buffer = Buffer.concat([buffer, Buffer.from(chunk)]);
        void drain();
    });

    async function drain(): Promise<void> {
        while (true) {
            const sep = buffer.indexOf('\r\n\r\n');
            if (sep < 0) return;
            const header = buffer.subarray(0, sep).toString('utf8');
            const match = /Content-Length:\s*(\d+)/i.exec(header);
            if (!match) {
                buffer = buffer.subarray(sep + 4);
                continue;
            }
            const length = Number(match[1]);
            const start = sep + 4;
            if (buffer.length < start + length) return;
            const body = buffer.subarray(start, start + length).toString('utf8');
            buffer = buffer.subarray(start + length);
            let msg: JsonRpc;
            try {
                msg = JSON.parse(body) as JsonRpc;
            } catch {
                continue;
            }
            await handle(msg);
        }
    }
}
