# idealis-coreの対応チェックリスト

チェック済みは共通の値・有限モデル・観測をつなぐホスト機能。検査エンジンの再実装とは区別する。未チェックは未対応または対象外で、実装予定を約束しない。

## 対応済み

- [x] Sessionのeffect/responseでModelとDriverを呼び出し、観測がモデルの許す候補に含まれるか照合する。
- [x] 保存Traceの再生と、matched / mismatch / invalidの区別。effect IDの不一致を拒否する。
- [x] version付きFiniteModelの読み込み・検査・準備と、観測付き遷移表への履歴照合。
- [x] 有限モデルに対する並行履歴の検査。unknown/cancelと確定したreturn/failを区別し、予算切れを成功にしない。
- [x] 状態付きmodel handleを再利用するprotocol。
- [x] 操作列の削除・依存を扱う縮小候補。SUTで同じ失敗が残るかの判断は呼び出し側が担当する。
- [x] SimulationHostの時計・乱数・通信・永続化とHostTapeの再生。
- [x] compile_tlaでrtla → rtla-compiler → rtlcを接続し、対応fragmentをnative/Wasmで検査する。
- [x] optionalなsymmetry.modelValuesを運び、Rust frontendとrtlcの限定SYMMETRYをnative/Wasmで扱う。適用形状・safety限定と宣言の前提はrtlcへ委譲し、一般の対称性証明やVIEWとは区別する。
- [x] check-safetyのoptionalなoptionsでworkers・symmetryActions・partialOrderをRustへ渡す。既存JSONは省略でき、未知のfieldと不正なworker数を拒否する。Wasmは逐次。限定した削減の前提・fallback・計数と最短反例の制限はrtlcへ委譲する。
- [x] compile-tla / check-tlaのmodulesへ明示した追加source集合を渡す。modulesを省略した既存要求と、要求間で異なる同名moduleを区別する。ファイルシステムへ依存しない。
- [x] model-value tagを含むportable値の正規化とnative/Wasmへの輸送。JSのfromPortableValueはmodel valueをtag付きobjectで返し、文字列へ変換しない。
- [x] rtlcのSafetyProgram / SafetyResult / Value / CoreError等の再export。
- [x] check-safety-coverageでSafetyCoverageReportをnative/Wasmへ返す。通常のsafety応答とは別のopt-in要求で、逐次のbranch計数とworkの内訳を保持する。
- [x] version 1のnative JSONLとホストimport不要のWasm境界。

## 非対応・対象外

- [ ] すべてのTLA+構文、独自のSMT求解、TLA+/SMTの汎用相互変換。
- [ ] OS/process/networkへの自動障害注入、実thread/awaitの自動計装、weak memoryのモデル化。
- [ ] Rust内での任意SUTのI/Oや各テストランナーの所有。
- [ ] 一致した有限履歴だけからの一般的なrefinement証明やliveness保証。
- [ ] 任意言語の埋め込みinvariantの注釈抽出・検証条件生成。

## 制約と根拠

- [x] モデルやDriverの効果は呼び出し側が評価する。Wasmコア自身はホストのI/O・時計・threadを要求しない。
- [x] 未完了・無効・予算切れを適合性の成功に読み替えない。

[README](README.md)、[protocolとSession](src/lib.rs)、[有限モデル](src/finite.rs)、[履歴](src/history.rs)、[ホスト](src/host.rs)、[回帰](tests/rtlc.rs)、[共有オラクル](../../docs/shared-oracle.md)を参照。
