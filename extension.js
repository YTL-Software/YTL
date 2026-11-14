const path = require('path');
const vscode = require('vscode');
const { LanguageClient, LanguageClientOptions, ServerOptions } = require('vscode-languageclient/node');

let client;

function activate(context) {
    // Показываем сообщение, что клиент активировался
    vscode.window.showInformationMessage('YTL Extension activated!');

    const serverPath = context.asAbsolutePath(path.join('ytl-lsp', 'target', 'debug', 'ytl-lsp.exe'));

    // Проверяем, существует ли файл сервера
    if (!require('fs').existsSync(serverPath)) {
        vscode.window.showErrorMessage(`Server not found at: ${serverPath}`);
        return;
    }

    const serverOptions = {
        run: { command: serverPath },
        debug: { command: serverPath }
    };

    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'ytl' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ytl') 
        },
        // Костыль для бага Content-Length в tower-lsp
        errorHandler: {
            error: () => ({ action: 1 }), // 1 = Continue (не закрывать соединение)
            closed: () => ({ action: 1 })
        }
    };

    client = new LanguageClient(
        'ytl-lsp',
        'YTL Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
    vscode.window.showInformationMessage('YTL LSP client started!');

    // =========================================================
    // КОМАНДА ДЛЯ ВЫБОРА ПОЛЕЙ (ВЫЗЫВАЕТСЯ ИЗ RUST)
    // =========================================================
    let disposable = vscode.commands.registerCommand('ytl.selectInheritedFields', async (uri, lineNumber, fields, indent) => {
        // 1. Формируем список для окна
        const items = fields.map(f => ({
            label: `${f[0]} ${f[1]}`, // Например: "speed 32"
            description: 'Inherited field',
            picked: true // По умолчанию ставим галочку на все
        }));

        // 2. Показываем окно с галочками и поиском
        const selected = await vscode.window.showQuickPick(items, {
            canPickMany: true,
            placeHolder: 'Select inherited fields to insert',
            title: 'YTL: Inherit Fields'
        });

        // Если юзер закрыл окно или ничего не выбрал - выходим
        if (!selected || selected.length === 0) {
            return;
        }

        // 3. Формируем текст для вставки 
        // ВАЖНО: Добавляем \n в самое начало, чтобы первое поле не прилипало к строке с блоком
        const textToInsert = "\n" + selected.map(s => `${indent}${s.label}\n`).join('');

        // 4. Вставляем в файл
        const edit = new vscode.WorkspaceEdit();
        // Преобразуем строку URI обратно в объект vscode.Uri
        const parsedUri = vscode.Uri.parse(uri); 
        
        // Вставляем на следующую строку после наследования (lineNumber + 1)
        const position = new vscode.Position(lineNumber + 1, 0);
        
        edit.insert(parsedUri, position, textToInsert);
        await vscode.workspace.applyEdit(edit);
    });

    context.subscriptions.push(disposable);
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = { activate, deactivate };