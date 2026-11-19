# Cacheの契約境界を修正する

PR #70のレビューをコードと突き合わせ、検査対象・実行者・保存する値の境界に回帰テストを追加した。cacheは既定で無効のままにする。[計画](../plans/cache-review.md)の条件を使い、Redの失敗も証拠へ保存した。

- optionsの余分なprogram/bounds/kindはrequestへ展開しない。位置引数のunsafeなprogramを検査する。
- TS helperとWasm adapterは無効時に通常checkを呼び、cacheをserialize/decodeしない。循環参照を含むcacheもそのまま返す。
- snapshotは正確なown fields、denseな配列、canonicalな値を要求する。digest対象のset順序をdecoderが書き換えない。
- receiptを実行時のowner/fence/runIdへ結び付ける。古いreceiptを新しいleaseへ付け替えられない。replacement workerは自分のleaseで実行する。
- JSONで符号を保持できない`-0`はlease取得前に拒否し、`+0`は受け付ける。
- digestが壊れたready entryをevictできる。revision CASとfenceを維持し、再取得で世代を進める。
- publishが所有権を解放した後はreleaseしない。staleの後のcleanup失敗は結果のcleanupErrorへ返す。実行とcleanupの両方が失敗した場合はAggregateErrorのcauseとerrorsへ元の例外を残す。
- Rustのcache用portable変換を、1展開64 KiBの保守的な予算へ制限する。共有値の重複を含むサイズを複製前に計算し、超過時はstreaming探索を続ける。途中までのsuccessor列を保存しない。満杯のcacheへの新規保存もstreamingへ戻す。

Rustの小さい共有木のRedでは、14件の展開をすべて保存してしまうことを再現した。修正後は保持できるprefixだけを保存する。深さ40と容量0・1・100でも、work以外は通常探索と同じbounded結果になる。初回テストのfield名誤りとTS型エラーは`red-*.log`へ保存し、挙動のRedは`red-*-02.log`と区別した。

Leanの故障対照は、通常の非ゼロ終了と、指定した定理内の失敗を要求する。32条件は5つのBoolean全組み合わせを重複なく含むことを先に検査する。評価manifestには値変換とQuintの設定ソースを含め、壊したpropertyが違反することも直接検査する。Criterion再生では中央値に加え、信頼区間の両端も照合する。

ローカルでは以下を確認した。

- `just check`、Node/Vitestのcache契約27件、Rust全targetとfmt/clippy。
- `just browser`の7件と`just portable-test`。配布したnpm/Cargoを使う独立consumer。
- Quintの12モデル検査、32再利用条件、評価と再生。
- Leanの27定理、7故障対照、23,606 policy遷移、Rust/Wasmの32条件。
- mutation score 95.4%。既存のsurvivorは残り、cache全体の変異検査率ではない。
- Jevの40比較とoffline再生。最初のexcerpt-budgetエラーは参照する関数を絞って解消し、checklistを更新した。

Criterionは9入力のuncached・cold・warm・JSON snapshotを各10標本で計測した。小さい6モデルではwarmも通常探索より1.11〜1.34倍遅い。以下は同じrun内の中央値で、他ツールとの速度比ではない。

| 入力              | 通常探索 µs | native warm µs | JSON snapshot µs | 通常 / warm |
| ----------------- | ----------: | -------------: | ---------------: | ----------: |
| derived-fold-16   |       15.22 |          11.95 |            25.78 |        1.27 |
| derived-fold-256  |       84.64 |          12.20 |            26.07 |        6.94 |
| derived-fold-4096 |     1193.65 |          12.06 |            25.55 |       98.94 |

36区間とsource/sampleのdigestを再生した。benchmarkのcompile中に契約検査・証明・browserを実行したが、26.96秒のcompileが終わった後の計測中には別の検証を起動していない。共有環境の単一runであり、広い信頼区間を含む。過去runの測定値は上書きしない。

証拠は[`cache-review`](../runs/cache-review/evidence.json)へ保存する。Leanの対象はpolicyと到達関係であり、Rust実装全体の機械証明ではない。64 KiBは1展開の変換予算で、cache全体のRSS上限ではない。
