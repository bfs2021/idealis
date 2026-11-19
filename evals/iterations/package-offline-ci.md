# 空のstoreで配布consumerを検査する

[PR #70のCI](https://github.com/mizchi/idealis/actions/runs/44570342940)は、最後の配布consumerのoffline installで失敗した。名前で指定した`@types/node@24.19.2`を解決するmetadataがpnpm storeになかった。型定義がローカルにある環境では通るため、既存storeの内容に依存していた。

consumer専用の空storeを指定すると、ローカルでも同じ段階で失敗した。修正後は配布したローカルtarballだけをofflineでインストールする。型検査用の`@types/node`と、その依存の`undici-types`は、lockfileで固定した開発依存からconsumerへコピーする。TypeScript compilerも既存と同じく開発依存を使う。公開packageへ新しい依存は加えない。

新しい空storeでも、TS/Wasmの実行、型検査、CLIの終了コード0/1/2、Rustと7 crateの独立consumerが成功した。`just check`とJevの40比較のoffline再生も通る。consumerを毎回新規作成するため、後続runも空storeから始まる。これは配布物の検査であり、cacheの意味論やRust全体の証明ではない。

Red、Green、CIの失敗箇所と変更ソースのdigestを[`package-offline-ci`](../runs/package-offline-ci/evidence.json)へ保存する。
