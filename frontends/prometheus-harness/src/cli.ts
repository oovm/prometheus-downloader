import { Command } from 'commander';
import { runMcpStdio } from './mcp.js';
import { createStrategy, listStrategies, testStrategy, workspaceRoot } from './tools.js';

export async function runCli(argv: string[]): Promise<number> {
    const program = new Command();
    program.name('prometheus-harness').description('Author Prometheus strategy plugins').version('0.0.1');

    program
        .command('mcp')
        .description('Start the MCP stdio server')
        .action(async () => {
            await runMcpStdio();
        });

    program
        .command('list')
        .description('List strategy plugins in the workspace')
        .action(async () => {
            const result = await listStrategies();
            process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
        });

    program
        .command('create')
        .argument('<platform>', 'strategy id (kebab-case)')
        .description('Write a strategy plugin scaffold')
        .action((platform: string) => {
            const result = createStrategy(platform);
            process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
        });

    program
        .command('test')
        .argument('<platform>', 'strategy id')
        .argument('<url>', 'url to inspect')
        .description('Run matches/inspect for a strategy')
        .action(async (platform: string, url: string) => {
            const result = await testStrategy(platform, url);
            process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
            if (!result.success) process.exitCode = 1;
        });

    program
        .command('workspace')
        .description('Print the resolved workspace root')
        .action(() => {
            process.stdout.write(`${workspaceRoot()}\n`);
        });

    await program.parseAsync(argv, { from: 'user' });
    const code = process.exitCode;
    return typeof code === 'number' ? code : 0;
}
