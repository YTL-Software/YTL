const path = require('path');
const fs = require('fs');
const vscode = require('vscode');
const { LanguageClient } = require('vscode-languageclient/node');

let client;

function activate(context) {
    // Определяем имя бинарника в зависимости от ОС
    const binaryName = process.platform === 'win32' ? 'ytl-lsp.exe' : 'ytl-lsp';
    
    // Автоматически находим путь к бинарнику внутри папки расширения
    const serverPath = context.asAbsolutePath(path.join('bin', binaryName));

    // Проверяем, существует ли файл сервера
    if (!fs.existsSync(serverPath)) {
        vscode.window.showErrorMessage(`YTL Server not found at: ${serverPath}`);
        return;
    }

    // Даем права на запуск для Linux/macOS
    if (process.platform !== 'win32') {
        try {
            fs.chmodSync(serverPath, 0o755);
        } catch (e) {
            console.error(`Failed to chmod server binary: ${e.message}`);
        }
    }

    const serverOptions = {
        run: { command: serverPath },
        debug: { command: serverPath }
    };

    const clientOptions = {
        documentSelector: ['ytl'],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ytl')
        },
        errorHandler: {
            error: () => ({ action: 1 }), // Continue
            closed: () => ({ action: 1 }) // DoNotRestart
        }
    };

    client = new LanguageClient(
        'ytl-lsp',
        'YTL Language Server',
        serverOptions,
        clientOptions
    );

    client.start().catch(err => {
        vscode.window.showErrorMessage(`YTL LSP Error: ${err.message}`);
    });

    // =========================================================
    // КОМАНДА ДЛЯ ВЫБОРА ПОЛЕЙ (ВЫЗЫВАЕТСЯ ИЗ RUST)
    // =========================================================
    const disposable = vscode.commands.registerCommand(
        'ytl.selectInheritedFields',
        async (uri, lineNumber, parentsWithFields, indent) => {
            
            // 1. ШАГ ПЕРВЫЙ: Выбор родителя (если их несколько)
            // parentsWithFields приходит от Rust как массив: [ [Имя, [ [Поле, Значение], ... ]], ... ]
            const parentItems = parentsWithFields.map(([parentName, fields]) => ({
                label: parentName,
                description: `${fields.length} fields available`,
                parentName: parentName
            }));

            let selectedParent = parentItems[0];

            if (parentItems.length > 1) {
                selectedParent = await vscode.window.showQuickPick(parentItems, {
                    placeHolder: 'Select parent to inherit from',
                    title: 'YTL: Choose Parent'
                });
                if (!selectedParent) return; // Пользователь нажал Esc
            }

            // 2. Находим поля выбранного родителя
            const parentData = parentsWithFields.find(([name]) => name === selectedParent.parentName);
            if (!parentData) return;
            
            const fields = parentData[1];

            // 3. ШАГ ВТОРОЙ: Выбор конкретных полей родителя
            const items = fields.map(f => ({
                label: `${f[0]} ${f[1]}`,
                description: 'Inherited field',
                picked: true
            }));

            const selected = await vscode.window.showQuickPick(items, {
                canPickMany: true,
                placeHolder: `Select fields from ${selectedParent.parentName} to insert`,
                title: 'YTL: Inherit Fields'
            });

            if (!selected || selected.length === 0) {
                return;
            }

            // 4. Вставляем выбранные поля в документ
            const textToInsert = '\n' + selected.map(s => `${indent}${s.label}\n`).join('');

            const edit = new vscode.WorkspaceEdit();
            const parsedUri = vscode.Uri.parse(uri);
            const position = new vscode.Position(lineNumber + 1, 0);

            edit.insert(parsedUri, position, textToInsert);
            await vscode.workspace.applyEdit(edit);
        }
    );

    context.subscriptions.push(disposable);
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = { activate, deactivate };