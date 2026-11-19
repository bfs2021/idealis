# 完了結果の再利用をLeanと実装で検査する

[事前計画](../plans/verified-cache.md)をa3bd5e1f2で固定した。基点はquint-regressionの7edd4d778。採用範囲は、同一backend build・requestに対する非空の`explicit-complete`を再利用する任意の層。入力が異なるモデルの部分探索の共有は含めない。[APIと前提](../../docs/check-cache.md)を参照。

コアの状態遷移をLeanとTSで別に実装した。TS coordinatorはそのpolicyをCASの前後で評価する。SQLiteの条件付き更新とHTTP adapterを用意し、plannerはhitと所有中のtaskを分け、missだけを既存の予算計画へ渡す。新しいmanifestのtask ID/sourceVersionと、結果を作ったrunの出所は分けて保存する。

## 証明と対照

Lean 4.35.0 / Stdだけで17定理を証明した。状態やtraceの長さには有限上限を置かない。全定理のaxiom一覧と`leanchecker`による再検査を保存した。利用するのは標準の`propext`、`Classical.choice`、`Quot.sound`で、graphに関する2定理はaxiomに依存しない。

| 検査           | 条件                                                               | 結果                                |
| -------------- | ------------------------------------------------------------------ | ----------------------------------- |
| Lean           | identity、世代、所有者、期限、receipt、complete、任意長trace、閉包 | 17定理を確認                        |
| Leanの故障対照 | fence、receipt identity、completeのguardを別々に除く               | 3件とも証明に失敗                   |
| TS / Lean差分  | 37到達状態 × 638操作、fence 0..4、時刻0..3                         | 23,606遷移が一致、12応答codeへ到達  |
| TLC正常版      | 2 worker、取得世代2、clock 0..3、seed 1                            | 626生成、220 distinct、不変条件成立 |
| TLC到達性対照  | 正常版で「readyに到達しない」を検査                                | 2操作でpublishへ到達                |
| TLC故障版      | fence、receipt、completeを別々に壊す                               | 3件とも反例、TSへ再生すると拒否     |

LeanはCAS自体やTSプログラム全体を証明していない。TSとの比較は有限領域での対応検査。storageのlinearizability、意味に影響する依存identityの網羅、backendの完全探索判定は前提に残る。graphの閉包定理は、将来の変更後モデルの再検査条件を定めるもので、細粒度の探索cacheを実装した証拠ではない。

最初のRedはcache moduleがない状態から始めた。Leanの予約語と証明分岐、TLA+モデルの代入右辺の括弧不足も修正した。括弧不足のTLC終了code 75を不変条件違反のcode 12として受理しなかった。work=0は既存backend契約が拒否するため、work=1で予算切れを検査する条件へ直した。配布consumerではmodule headerとNode型設定を修正した。これらを未知の実プロジェクト故障の追加検出には数えない。

## 実装の検査と費用

node:test / Vitestで共有する11テストを追加した。実Wasmの完全探索・空初期集合・反例・型エラー・予算切れ、入力のsnapshot、digest破損、期限・renew・eviction後のABA、同時取得、別connection、HTTPと再起動を扱う。SQLiteにleaseを書いた子processをSIGKILLし、期限前のbusy、期限後の再取得と古い結果の公開拒否も確認した。

変更していないQuint生成物6件を実Wasmで検査した。冷えたstoreでは6回探索し、SQLiteを閉じて新しいclientで再開すると6件ともhitし、探索呼び出しは0回。状態数はTLC保存オラクルと同じ`4, 1, 1, 3, 3, 1`。同じsource/binaryでの再生も一致した。Chromiumでも別client・新しいWasm instanceからHTTP経由で同じ結果を再利用した。

1回のNode観測では、6件分のtask lookup/実行はcold 31.38 ms、warm 6.17 msだった。Wasm初期化とsource compileは除いている。同時に他の開発検査を動かしており、反復・分散・大規模モデルを測っていないため高速化倍率は採用指標にしない。今回確認した効果は同一taskの探索呼び出しを省けること。canonical化・identity検査・store輸送の費用は残る。

`just check`は既存Vitest/node各77件、`cache-test`は各11件、`just browser`は6件が成功した。既存mutationは104 kill / 5 survive / 133 compile error、score 95.4%。このscoreを新しいcache全体のmutation coverageとは扱わない。cacheのguardには上記の独立した証明故障対照とTLC反例を用いた。

配布archiveの独立consumerで、新しい3 entry、実WasmとSQLiteのcold/hit、TSの型、既存Rust crate/CLIを検査した。評価ハーネスと反例再生、既存のdevelopment/confirmation loopと10 witnessの再生、Jevの7 crateに対するlive/offline比較も成功した。GitHub CIにはLean・参照機比較とNode/Vitestを接続したが、このローカルbranchのremote CIは未実行。

正常版は再利用を可能にし、人工故障は拒否できた。task単位の任意cacheとして採用する。状態単位のNext cacheへ進む前に、値の安定した永続形式、変更したpropertyの全状態への再評価、現在のNextに対する閉包検査と到達性の扱いを実装する。liveness、悪意あるworker、任意backend、全モデルでの性能改善は今回の保証範囲に含めない。

入力・digest・証明監査・失敗ログ・TLC反例・実測と再生は[保存索引](../runs/verified-cache/evidence.json)に残す。実行手順は[利用ガイド](../../docs/check-cache.md#証明する範囲)を参照。
