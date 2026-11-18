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
- [x] 関数・record・sequenceのimmutable domainを共有し、64要素以下の全Boolまたは全Int(0/1)を別tagのbit列に保持する。混在・範囲外・65要素以上はScalar列へ戻す。公開Value/IRとcanonical identity、workを維持する。
- [x] domainと関数のhash衝突を完全比較し、worker新規domainのkey/payload Refをrootへremapする。root所属domainはimmutableに共有し、所属metadataを等値性へ含めない。
- [x] packed関数の型と完全に一致する有限rangeのmembershipでは、domain確認後に同じ要素数の論理workを一括計上する。workerのbudget付きoverlayは既存の要素順へ戻す。通常の関数検査も値だけのowned iteratorを使う。
- [x] native forkはroot domainが16個以下なら、64 key以下のdomainにworker専用wrapperを持ち、参照カウンタを分離する。merge後はcanonical rootへ戻し、対象外は通常のArc共有へ戻す。

- [x] 汎用Scalar列の関数更新は、コピー前のborrowed viewでbase/localの既存値を検索する。hash衝突はdomain/payloadを完全比較し、新規の値だけを確保する。packed値とpackedへの復帰、workと深さの検査を維持する。

- [x] packed関数の内部hashはdomain/payload/型saltをfmix64で混合する。公式C++の7出力、固定4,096値の分布、Bool/Intの衝突で完全比較とcanonical identityを検査する。汎用関数のhashと公開Value/IRは維持する。

- [x] immutableなprogramのData式をslotへ登録し、Arenaごとに初回成功後のScalarをcacheする。未評価分岐の値を先に評価せず、失敗・状態依存式をcacheしない。未登録の式は従来の評価へ戻り、workerのlocal Refはmergeでremapする。

## TLC互換の追加対応

- [x] model valueを文字列と区別して保持・比較し、typed model valueの不正な等価比較を評価エラーにする。portable値のtagはmodel-value。
- [x] 整数のfloor division、正の除数によるmodulo、非負指数のpower。ゼロ除算・0^0・i32 overflowは評価エラー。
- [x] 有限domainの式CHOOSE。正規化した順序の最初の一致を選び、tuple pattern、短絡と無解時エラーを扱う。Step::Chooseによる状態の候補代入とは別。
- [x] state/action constraintで探索を制限する。除外する境界候補もinvariantと対応するpropertyを検査し、除外をdeadlockとは扱わない。completeは制約後のグラフに対する結果。
- [x] 時間論理のforall/existsを定数の有限domainで展開する。tuple、空domain、入れ子を扱い、展開もwork予算を消費する。
- [x] temporal展開の訪問済み式は256式以下で固定長bit集合に保持し、それより大きいpoolはHashSetへ戻す。todo/next/pendingの順序とworkのtickは変えない。
- [x] 単一の2〜64個のuntyped model value群に対する限定SYMMETRY。全群をdomainに持つ関数のInt/Boolまたはflat recordと、node部分集合を扱う。recordのfield名・Int/Bool型を関数内で揃える。同じnodeの関数値・集合所属を同じ行として完全比較する。個別の群要素・順序依存式と未対応shapeは拒否する。
- [x] 同一視用keyと具体状態を分離し、元の具体状態から探索・traceを作る。workへkey構築を計上する。宣言がInit/Next/invariantを保つことは利用者の前提で、gateをequivariance証明とは扱わない。
- [x] SafetyOptionsのsymmetry_actionsはopt-in。先頭に単一node bindingを持つbranchの同じ行を代表だけ展開する。Init・複数binding・tuple binding・全群以外のdomainは通常展開。generated/workは削減後の計数。
- [x] nativeのsymmetryは64状態以上のfrontierを並列展開する。rootへのRef移送後に逐次にkeyを照合し、代表展開とも併用する。狭いfrontierとWasmは逐次へ戻す。
- [x] SafetyOptionsのpartial_orderはopt-inで逐次実行する。invariantと他branchが参照せず、他branchが戻せないBool flagのFALSE→TRUE操作だけを先に実行する。依存・未知の式・不正な状態型では通常展開へ戻す。temporal/fairness・constraint・initial/action propertyでは通常探索、SYMMETRYとの併用は拒否する。元のグラフの最短反例は保証しない。
- [ ] 一般のSYMMETRY、複数群、typed model value、node参照やrecord内のnested payload、temporal/fairness・constraint・initial/action property・coverageとの併用。
- [ ] 任意の独立性解析や循環に対応する一般のPOR。現在の限定PORの条件外への適用。
- [ ] VIEWによる状態の同一視。
- [ ] ディスクへの状態・queueの退避とcheckpoint/resume。
- [ ] seed付きのIR simulation、DFID探索。
- [x] check_with_coverageの逐次action coverage。initial/nextのIR branch順を保ち、attempts、enabled、generated、distinct、受理時の検査を含むworkとunattributedWorkを返す。resultとwork予算は通常checkと一致する。
- [ ] 式・source locationごとのcoverage、TLCの全cost model、並列coverage。
- [ ] TLC CLIの全オプションと反例出力形式。ALIASはcompiler/host側の未対応項目。

## 移植性・追加保証と対象外

- [ ] TLA+の構文解析・名前解決・IRへのコンパイル、TLC CLI全体の互換性。
- [ ] 無限状態の完全探索、任意のTLA+構文やSMT理論。TLC互換性の不足とは別に扱う。
- [ ] Wasm内部の探索マルチスレッド、temporal/fairness探索の内部並列化。
- [ ] 型別命令VM、訪問済み照合やstate受理の内部並列化。0059の試作と0061の命令IDによる直接実行は後退があり不採用。
- [ ] liveness反例の最短性、fairnessを満たすbehaviorの存在の別証明。
- [ ] このcrate単体のブラウザJSON ABI。Wasm境界はidealis-coreが提供する。

## 制約と根拠

- [x] complete / bounded / empty / 各違反 / 評価エラーを区別する。boundedやemptyを安全の証明にしない。
- [x] workはwall timeや厳密なメモリ上限ではない。並列探索のworkと予算境界の部分結果は逐次と同じとは限らない。
- [x] 集合foldの文字列・record、関数key幅等の制限を維持する。model valueを含む集合foldも未対応。汎用Valueのvariantの存在だけで対応とは数えない。

[README](README.md)、[IR](src/ir.rs)、[探索](src/safety.rs)、[関数値](src/safety_functions.rs)、[値の登録・移送](src/safety_values.rs)、[時間的性質](src/temporal.rs)、[定数の準備](src/safety_prepared.rs)、[限定symmetry](src/safety_symmetry.rs)、[適用条件](../../docs/symmetry.md)、[並列探索](src/safety_parallel.rs)、[回帰](tests/parallel_safety.rs)、[有限対応の比較入力](../rtla-compiler/tests/finite-support.json)、[対応fragment](../../docs/rust-safety.md)を参照。

[限定POR](src/safety_partial_order.rs)と[optionの制限](../../docs/reductions.md)を参照。
