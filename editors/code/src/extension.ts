import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    const abyssCompletionProvider = vscode.languages.registerCompletionItemProvider('abyss', {
        provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
            const keywords = [
                'forge',
                'unveil',
                'oracle',
                'transmute',
                'orbit',
                'revolve',
                'eject',
                'reveal',
                'morph',
                'engrave',
                'summon',
                'ward',
                'fate',
                'augury',
                'bless',
                'curse',
                'manifest',
                'naught',
                'incant',
                'perish',
                'invocation',
                'aura'
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
