# rtlcの対応チェックリスト

チェック済みはコンパイル済みの有限検査IRに対する機能。TLA+ソースの受理とは区別する。未チェックは未対応または対象外で、実装予定を約束しない。

## 対応済み

- [x] SafetyProgramからsuccessorを生成し、BFSで状態を重複排除する。hash衝突時にも状態全体を比較する。
- [x] invariant・deadlock・初期state property・action property・定数ASSUMEの検査とsafety反例の復元。
- [x] 対応するtemporal formula、stuttering、WF/SFとlasso反例。safetyと異なり最短lassoは保証しない。
- [x] signed 32-bit整数、Bool、文字列、有限集合・有限関数。recordとsequenceも有限関数として等価性を判定する。
- [x] IRの算術・集合・関数・record・sequence操作、有限量化、EXCEPTと@、foldの評価。
- [x] 未一致CASEの評価時エラーと、singletonの文字列domainに限定したvariant-tag。
- [x] select-seq / sort-seqのcallback IRとassert。選択は順序・重複を保ち、sortはTLCと同じ挿入順で新要素と直前要素を比較する。callbackのslotを検査し、反復・比較でworkを消費する。assertはmessageも評価する。
- [x] maxDepth/maxStates/maxWorkとtemporal積グラフの予算を扱う。
- [x] nativeのcheck_parallelは1〜256 worker上限を受け取り、適したsafety frontierを並列展開する。
- [x] 軽いscalar・狭いfrontier・action/temporal property・fairnessは逐次へfallbackする。Wasmは逐次check。
- [x] Rust rlibとしてnative/Wasmへビルドでき、idealis-coreやparser、外部solverには依存しない。

## TLC互換の追加対応

- [x] model valueを文字列と区別して保持・比較し、typed model valueの不正な等価比較を評価エラーにする。portable値のtagはmodel-value。
- [x] 整数のfloor division、正の除数によるmodulo、非負指数のpower。ゼロ除算・0^0・i32 overflowは評価エラー。
- [x] 有限domainの式CHOOSE。正規化した順序の最初の一致を選び、tuple pattern、短絡と無解時エラーを扱う。Step::Chooseによる状態の候補代入とは別。
- [x] state/action constraintで探索を制限する。除外する境界候補もinvariantと対応するpropertyを検査し、除外をdeadlockとは扱わない。completeは制約後のグラフに対する結果。
- [x] 時間論理のforall/existsを定数の有限domainで展開する。tuple、空domain、入れ子を扱い、展開もwork予算を消費する。
- [ ] SYMMETRYによる状態削減。活性検査への適用条件も必要。
- [ ] VIEWによる状態の同一視。
- [ ] ディスクへの状態・queueの退避とcheckpoint/resume。
- [ ] seed付きのIR simulation、DFID探索。
- [ ] アクション・式ごとのcoverageと評価コストの計測。
- [ ] TLC CLIの全オプションと反例出力形式。ALIASはcompiler/host側の未対応項目。

## 移植性・追加保証と対象外

- [ ] TLA+の構文解析・名前解決・IRへのコンパイル、TLC CLI全体の互換性。
- [ ] 無限状態の完全探索、任意のTLA+構文やSMT理論。TLC互換性の不足とは別に扱う。
- [ ] Wasm内部の探索マルチスレッド、temporal/fairness探索の内部並列化。
- [ ] liveness反例の最短性、fairnessを満たすbehaviorの存在の別証明。
- [ ] このcrate単体のブラウザJSON ABI。Wasm境界はidealis-coreが提供する。

## 制約と根拠

- [x] complete / bounded / empty / 各違反 / 評価エラーを区別する。boundedやemptyを安全の証明にしない。
- [x] workはwall timeや厳密なメモリ上限ではない。並列探索のworkと予算境界の部分結果は逐次と同じとは限らない。
- [x] 集合foldの文字列・record、関数key幅等の制限を維持する。model valueを含む集合foldも未対応。汎用Valueのvariantの存在だけで対応とは数えない。

[README](README.md)、[IR](src/ir.rs)、[探索](src/safety.rs)、[時間的性質](src/temporal.rs)、[並列探索](src/safety_parallel.rs)、[回帰](tests/parallel_safety.rs)、[有限対応の比較入力](../rtla-compiler/tests/finite-support.json)、[対応fragment](../../docs/rust-safety.md)を参照。
