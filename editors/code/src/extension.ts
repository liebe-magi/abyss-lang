import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    const abyssCompletionProvider = vscode.languages.registerCompletionItemProvider('abyss', {
        provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
            const keywords = [
                'forge',
                'unveil',
                'oracle',
                'trans',
                'orbit',
                'resume',
                'eject',
                'reveal',
                'morph',
                'engrave',
                'summon'
            ];
            const completionItems = keywords.map(keyword => {
                return new vscode.CompletionItem(keyword, vscode.CompletionItemKind.Keyword);
            });
            return completionItems;
        }
    });

    context.subscriptions.push(abyssCompletionProvider);
}

export function deactivate() { }
