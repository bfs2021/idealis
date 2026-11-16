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
- [x] nativeのcheck_parallelは1〜256 worker上限を受け取り、適したsafety frontierを並列展開する。
- [x] 軽いscalar・狭いfrontier・action/temporal property・fairnessは逐次へfallbackする。Wasmは逐次check。
- [x] Rust rlibとしてnative/Wasmへビルドでき、idealis-coreやparser、外部solverには依存しない。

## 非対応・対象外

- [ ] TLA+の構文解析・名前解決・IRへのコンパイル、TLC CLI全体の互換性。
- [ ] 無限状態の完全探索、任意のTLA+構文やSMT理論。
- [ ] Wasm内部の探索マルチスレッド、temporal/fairness探索の内部並列化。
- [ ] liveness反例の最短性、fairnessを満たすbehaviorの存在の別証明。
- [ ] このcrate単体のブラウザJSON ABI。Wasm境界はidealis-coreが提供する。

## 制約と根拠

- [x] complete / bounded / empty / 各違反 / 評価エラーを区別する。boundedやemptyを安全の証明にしない。
- [x] workはwall timeや厳密なメモリ上限ではない。並列探索のworkと予算境界の部分結果は逐次と同じとは限らない。
- [x] 集合foldの文字列・record、関数key幅等の制限を維持する。汎用Valueのvariantの存在だけで対応とは数えない。

[README](README.md)、[IR](src/ir.rs)、[探索](src/safety.rs)、[時間的性質](src/temporal.rs)、[並列探索](src/safety_parallel.rs)、[回帰](tests/parallel_safety.rs)、[対応fragment](../../docs/rust-safety.md)を参照。
