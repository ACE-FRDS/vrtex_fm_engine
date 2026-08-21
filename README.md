# Vertex FM Engine

FileMaker Pro の XML Clipboard を扱うためのデスクトップ開発支援 IDE です。

## Documentation

- [Vertex FM ENGINE 技術仕様書](docs/Vertex%20FM%20ENGINE%20技術仕様書.md)

## 実装済みの基盤

- Windows FileMakerカスタムClipboard形式（4-byte Little Endianヘッダー）の読込・書込
- Vue 3 + TypeScript + Quasar + Pinia
- Monaco XML Editor
- SQLite履歴・ライブラリ・お気に入り・メモ・編集リビジョン
- XML形式判定・検証・スクリプトプレビュー
- FileMakerプロセス検出
- ブラウザ確認用フォールバック（ネイティブ機能はTauri実行時のみ有効）

## Development

前提: Node.js 20以降、Rust stable（MSVC toolchain）、WebView2、pnpm。

```bash
pnpm install
pnpm run dev
pnpm run tauri dev
```

フロントエンドのみの確認は `pnpm run dev`、FileMaker ClipboardとSQLiteを含むデスクトップアプリは `pnpm run tauri dev` を使用します。

```bash
pnpm run build
cd src-tauri
cargo test
```
