import * as path from 'path';
import { workspace, ExtensionContext, window } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // Get the server path from configuration or use default
    const config = workspace.getConfiguration('wtlang');
    let serverPath = config.get<string>('server.path') || 'wtlang-lsp';

    // If serverPath is not absolute, try to find it in workspace or PATH
    if (!path.isAbsolute(serverPath)) {
        // Try to find the server in the workspace target directory
        const workspaceFolders = workspace.workspaceFolders;
        if (workspaceFolders && workspaceFolders.length > 0) {
            const workspaceRoot = workspaceFolders[0].uri.fsPath;
            const debugPath = path.join(workspaceRoot, 'target', 'debug', 'wtlang-lsp.exe');
            const releasePath = path.join(workspaceRoot, 'target', 'release', 'wtlang-lsp.exe');
            
            const fs = require('fs');
            if (fs.existsSync(releasePath)) {
                serverPath = releasePath;
            } else if (fs.existsSync(debugPath)) {
                serverPath = debugPath;
            }
        }
    }

    const run: Executable = {
        command: serverPath,
        options: {
            env: process.env,
        },
    };

    const serverOptions: ServerOptions = {
        run,
        debug: run,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'wtlang' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.wt'),
        },
    };

    client = new LanguageClient(
        'wtlang',
        'WTLang Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    window.showInformationMessage('WTLang Language Server activated');
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
