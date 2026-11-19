# idealis-coreの対応チェックリスト

チェック済みは共通の値・有限モデル・観測をつなぐホスト機能。検査エンジンの再実装とは区別する。未チェックは未対応または対象外で、実装予定を約束しない。

## 対応済み

- [x] source APIとnative/Wasmのcompile-tla / check-tlaで`irVersion`を明示指定できる。省略・明示1はv1、明示2はexpression定義を共有するv2を生成・実行する。明示2でモジュール連結や設定の上書きを要する入力は、v1と共通の連結と検査が通った場合にprotocol拒否にする。

- [x] Sessionのeffect/responseでModelとDriverを呼び出し、観測がモデルの許す候補に含まれるか照合する。
- [x] 保存Traceの再生と、matched / mismatch / invalidの区別。effect IDの不一致を拒否する。
- [x] version付きFiniteModelの読み込み・検査・準備と、観測付き遷移表への履歴照合。
- [x] 有限モデルに対する並行履歴の検査。unknown/cancelと確定したreturn/failを区別し、予算切れを成功にしない。
- [x] 状態付きmodel handleを再利用するprotocol。
- [x] 操作列の削除・依存を扱う縮小候補。SUTで同じ失敗が残るかの判断は呼び出し側が担当する。
- [x] SimulationHostの時計・乱数・通信・永続化とHostTapeの再生。
- [x] compile_tlaでrtla → rtla-compiler → rtlcを接続し、対応fragmentをnative/Wasmで検査する。
- [x] compile-tla / check-tlaのmodulesへ明示した追加source集合を渡す。modulesを省略した既存要求と、要求間で異なる同名moduleを区別する。ファイルシステムへ依存しない。
- [x] model-value tagを含むportable値の正規化とnative/Wasmへの輸送。JSのfromPortableValueはmodel valueをtag付きobjectで返し、文字列へ変換しない。
- [x] rtlcのSafetyProgram / SafetyResult / Value / CoreError等の再export。
- [x] version 1のnative JSONLとホストimport不要のWasm境界。
- [x] check-safety-incremental / incremental-safetyでrtlcの状態別Next cacheを入出力し、merge-safety-expansionsで同一namespaceのsnapshotを統合する。Init・property・constraintsの再検査、保持状態数上限とhit/miss/writeを引き継ぐ。分散BFSやqueueのcheckpoint再開は行わない。

- [x] safety IR v2の定義共有・caller lexical frame付き遅延引数をrtlcで検証・実行する。request envelopeとWasm ABIはversion 1のままで、v2へ自動再試行しない。

## 非対応・対象外

- [ ] irVersion 2でのmodulesの連結と設定の上書き（ユーザーmoduleのEXTENDS・INSTANCE、operator置換・定数表・定義の上書き）。v1と省略は従来どおり受理する。
- [ ] すべてのTLA+構文、独自のSMT求解、TLA+/SMTの汎用相互変換。
- [ ] OS/process/networkへの自動障害注入、実thread/awaitの自動計装、weak memoryのモデル化。
- [ ] Rust内での任意SUTのI/Oや各テストランナーの所有。
- [ ] 一致した有限履歴だけからの一般的なrefinement証明やliveness保証。
- [ ] 任意言語の埋め込みinvariantの注釈抽出・検証条件生成。

## 制約と根拠

- [x] モデルやDriverの効果は呼び出し側が評価する。Wasmコア自身はホストのI/O・時計・threadを要求しない。
- [x] 未完了・無効・予算切れを適合性の成功に読み替えない。

[README](README.md)、[protocolとSession](src/lib.rs)、[有限モデル](src/finite.rs)、[履歴](src/history.rs)、[ホスト](src/host.rs)、[回帰](tests/rtlc.rs)、[共有オラクル](../../docs/shared-oracle.md)を参照。

- [x] 明示v2の非共有anonymous expression/action/temporal frameを扱い、callerのbinder・old・引数を遅延参照する。入口と引数要求へ各1 workを課金し、v1・既存上限を維持する。LET本体の自由な`@`は型付きprotocol拒否とする。
