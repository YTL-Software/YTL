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

    // Читаем настройку языка из конфигурации VS Code
    const language = vscode.workspace.getConfiguration('ytl').get('language', 'en');

    const clientOptions = {
        documentSelector: ['ytl'],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ytl')
        },
        initializationOptions: {
            ytl_language: language
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
                if (!selectedParent) return;
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

    // =========================================================
    // КОМАНДА ДЛЯ ПЕРЕКЛЮЧЕНИЯ ЯЗЫКА (EN/RU)
    // =========================================================
    const switchLangCmd = vscode.commands.registerCommand('ytl.switchLanguage', async () => {
        const config = vscode.workspace.getConfiguration('ytl');
        const currentLang = config.get('language', 'en');
        const newLang = currentLang === 'en' ? 'ru' : 'en';
        
        await config.update('language', newLang, vscode.ConfigurationTarget.Global);
        
        // 1. Останавливаем старый клиент
        if (client) {
            await client.stop();
            client = undefined;
        }

        // 2. Читаем новый язык
        const newLanguage = vscode.workspace.getConfiguration('ytl').get('language', 'en');
        
        // 3. Создаем новый clientOptions с новым языком
        const newClientOptions = {
            documentSelector: ['ytl'],
            synchronize: {
                fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ytl')
            },
            initializationOptions: {
                ytl_language: newLanguage
            },
            errorHandler: {
                error: () => ({ action: 1 }),
                closed: () => ({ action: 1 })
            }
        };

        // 4. Запускаем новый клиент
        client = new LanguageClient(
            'ytl-lsp',
            'YTL Language Server',
            serverOptions,
            newClientOptions
        );
        
        await client.start();

        // 5. Локализованное уведомление
        const message = newLanguage === 'ru' 
            ? "YTL: Язык переключен на RU" 
            : "YTL: Language switched to EN";
        vscode.window.showInformationMessage(message);

        // 6. Принудительно обновляем все открытые файлы, чтобы стереть старые ошибки
        vscode.workspace.textDocuments.forEach(async doc => {
            if (doc.languageId === 'ytl') {
                await vscode.commands.executeCommand('workbench.action.files.save', doc.uri);
            }
        });
    });

    // Создаем кнопку в статус-баре (внизу справа)
    const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = 'ytl.switchLanguage';
    statusBarItem.tooltip = 'Переключить язык YTL (EN/RU)';
    
    const updateStatusBar = () => {
        const lang = vscode.workspace.getConfiguration('ytl').get('language', 'en');
        statusBarItem.text = `$(globe) YTL: ${lang.toUpperCase()}`;
    };
    
    updateStatusBar();
    statusBarItem.show();

    // Слушаем изменения конфигурации для обновления кнопки
    vscode.workspace.onDidChangeConfiguration(e => {
        if (e.affectsConfiguration('ytl.language')) {
            updateStatusBar();
        }
    });

    context.subscriptions.push(disposable);
    context.subscriptions.push(switchLangCmd);
    context.subscriptions.push(statusBarItem);
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = { activate, deactivate };