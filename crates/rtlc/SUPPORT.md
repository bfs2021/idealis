# rtlcの対応チェックリスト

チェック済みはコンパイル済みの有限検査IRに対する機能。TLA+ソースの受理とは区別する。未チェックは未対応または対象外で、実装予定を約束しない。

## 対応済み

- [x] SafetyProgramからsuccessorを生成し、BFSで状態を重複排除する。hash衝突時にも状態全体を比較する。
- [x] invariant・deadlock・初期state property・action property・定数ASSUMEの検査とsafety反例の復元。
- [x] 対応するtemporal formula、stuttering、WF/SFとlasso反例。safetyと異なり最短lassoは保証しない。
- [x] signed 32-bit整数、Bool、文字列、有限集合・有限関数。recordとsequenceも有限関数として等価性を判定する。
- [x] IRの算術・集合・関数・record・sequence操作、有限量化、EXCEPTと@、foldの評価。
- [x] 未一致CASEの評価時エラーと、singletonの文字列domainに限定したvariant-tag。
- [x] maxDepth/maxStates/maxWorkとtemporal積グラフの予算を扱う。
- [x] safety IR v1を維持し、明示v2のexpression定義表とcall / parameter / captureを検証・評価する。定義graphの循環、ID、arity、capture、frame内slot、phase違反を拒否する。
- [x] v2の引数はcallerのlexical frameを保持し、要求時のcurrent / next / primeで毎回評価する。値cacheを持たず、call entryと引数評価に各1 workを課金し、枯渇をbounded/workにする。
- [ ] 再帰定義、action template共有。ソースからのv2生成は両frontendが担当し、このcrateのIR検証・実行とは分離する。
- [x] nativeのcheck_parallelは1〜256 worker上限を受け取り、適したsafety frontierを並列展開する。
- [x] 軽いscalar・狭いfrontier・action/temporal property・fairnessは逐次へfallbackする。Wasmは逐次check。
- [x] Rust rlibとしてnative/Wasmへビルドでき、idealis-coreやparser、外部solverには依存しない。
- [x] 任意のcheck_incrementalで、同じbuild・Next・全definition・変数順とportable状態について、完了したsuccessor列だけを再利用する。Init・property・constraints・deadlock・temporal/fairnessは毎回検査し、visitedや過去の合否を引き継がない。保持状態数の上限、snapshotのSHA-256検査と同じnamespaceのmerge・競合拒否を扱う。この経路は逐次探索で、分散BFSやcheckpoint再開ではない。
- [x] ExpansionOptions.enabledは既定false。無効なら通常のcheckへ委譲し、cacheを参照・更新せず、今回のhit/miss/writeを0とする。再利用には明示trueとbuild digest・保持上限を渡す。

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
- [x] v2の型所属はidentity range map-setを列挙せず判定する。通常値のmap-setとv1の評価は列挙を維持する。
- [x] v2の動的評価の入れ子は128までで、超過はprotocolエラー。定義内expression深さ48や既存v1の制限とは別に扱う。
- [x] workはwall timeや厳密なメモリ上限ではない。並列探索のworkと予算境界の部分結果は逐次と同じとは限らない。
- [x] 集合foldの文字列・record、関数key幅等の制限を維持する。model valueを含む集合foldも未対応。汎用Valueのvariantの存在だけで対応とは数えない。

[README](README.md)、[IR](src/ir.rs)、[v2契約](../../docs/safety-ir-v2.md)、[定義検証](src/safety_definitions.rs)、[探索](src/safety.rs)、[時間的性質](src/temporal.rs)、[並列探索](src/safety_parallel.rs)、[回帰](tests/parallel_safety.rs)、[有限対応の比較入力](../rtla-compiler/tests/finite-support.json)、[対応fragment](../../docs/rust-safety.md)を参照。

v2のmembership descriptorは条件分岐の選択枝へ用途を保ち、未選択枝を評価しない。通常の値評価とv1の列挙は維持する。

- [x] 明示v2の非共有anonymous expression/action/temporal frameを扱い、callerのbinder・old・引数を遅延参照する。入口と引数要求へ各1 workを課金し、v1・既存上限を維持する。LET本体の自由な`@`は型付きprotocol拒否とする。
