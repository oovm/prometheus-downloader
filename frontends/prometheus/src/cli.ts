import { Command } from 'commander';
import { download, info, version } from './index.js';

export async function runCli(argv: string[]): Promise<number> {
    const program = new Command();
    program.name('prometheus').description('Prometheus media acquisition').version(version(), '-V, --version');

    program
        .command('info')
        .argument('<url>', 'media URL')
        .description('Print media metadata as JSON')
        .action(async (url: string) => {
            const media = await info(url);
            process.stdout.write(`${JSON.stringify(media, null, 2)}\n`);
        });

    program
        .command('download')
        .argument('<url>', 'media URL')
        .requiredOption('-o, --output <dir>', 'output directory')
        .description('Download media into a directory')
        .action((url: string, opts: { output: string }) => {
            const result = download(url, opts.output);
            process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
        });

    program
        .command('mcp')
        .description('Start the MCP stdio server')
        .action(() => {
            process.stderr.write('MCP server is not available in this build.\n');
            process.exitCode = 1;
        });

    await program.parseAsync(argv, { from: 'user' });
    const code = process.exitCode;
    return typeof code === 'number' ? code : 0;
}
