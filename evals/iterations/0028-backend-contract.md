# 0028: 検査方式の保証をbackend契約で区別する

[`@mizchi/modelt/backend`](../../docs/backend-contract.md)へ、完全探索、有界の反例なし、帰納性、個別obligationの成立を区別する契約を追加した。[計画](../plans/0028-backend-contract.md)を実装前にコミットし、共有テストのRedから実装した。`SafetyResult`、SMTのsat/unsat API、Rust/Wasm ABIは変更していない。

## 契約で防ぐ取り違え

各結果はmodel/property ID、意味論profile、仮定一覧、backend ID/version、queryを保持する。wire decoderは矛盾したqueryと結果、不正なtrace、未知のfield/versionを拒否する。rtlcのdepth/state/work/temporal打切りは`inconclusive`とし、BMCのk以下の完全なcoverageへ昇格させない。

帰納性の集約は、同じ対象のinit/stepを要求する。候補不変条件とpropertyが異なる場合はimplicationも要求する。初期集合が非空と確認できず、必要なobligationが欠け、またはunknown/error/反例がある場合は未確定とし、個別の結果を残す。帰納性のstepに対するassignmentは、元のInitから到達するbehaviorの反例と区別した。

## 実応答との照合

[開発run](../runs/0028-backend-contract/result.json)の後、実Wasmの検査を専用integration testへ分離した。既存CIがWasmをbuildする前にも、Node/Vitestの共有テストを実行できる構成にした。さらにlassoを閉じるedgeもBMCのstep数に含める検査を追加した。最終実装の照合は[別のrun](../runs/0028-backend-contract-final/result.json)に保存し、開発runは編集していない。

| 検査                          | 結果と意味                                                                     |
| ----------------------------- | ------------------------------------------------------------------------------ |
| 同じ6条件のrtlc native/Wasm   | 6組の結果が一致。complete/資源切れ/違反を区別                                  |
| 0027のApalache保存応答        | 24応答を有界の結果とbehaviorへ変換し、wireを往復                               |
| 保存した帰納性のinit/step     | 同じ16状態ringに対する2条件を集約し`inductive`                                 |
| initだけの対照                | `inconclusive/obligations`。不足したstepを補わない                             |
| 個別の実Wasm integration test | empty、ASSUME違反、3種の予算切れ、protocol error、未対応queryとscope違いも確認 |

0027の元ファイルのdigestを検査してから応答を利用した。保存したpilotの初期集合の非空性は、具体的な列挙で確認した。NoErrorだけから非空性を推定する変換ではない。保存した帰納性のstepは候補不変条件からの保存検査であり、元モデルの有界検査と取り違えない。

## 検証と採用範囲

共有の契約テスト5件を含む全76件がVitestとnode:testで通過した。実Wasm integration、既存browser 2件、core/parserの実Chromium 2件、portable test、単独package consumer、既存evaluate/replayも通過した。package consumerはsolverなしで`/backend`の型を検査し、実Wasmの反例を同じ契約で取得する。[コマンドとログ](../runs/0028-backend-verification/result.json)を保存した。

共有テスト変更に伴う既存mutationのscoreは95.4%、104 killed / 5 survived / 133 compile errorsだった。対象は従来のconformanceとdurable-counterで、新しいbackend実装のmutation coverageではない。残る5件もログへ保存した。

共通契約とrtlc bridgeを採用する。metadataの一致とwire検査は、backendやSMT encodingの正当性を検証するproof certificateではない。今回はApalacheの汎用process/RPC adapter、SMT encoding、ITF importer、実SUTの未知故障発見、新しい性能測定を追加していない。これらは独立した実装と評価を必要とする。
