# 0033: idealisへ名前を揃える

[計画](../plans/0033-rename-idealis.md)に従い、プロジェクトとCLIをidealis、npmを`@mizchi/idealis`、Rustのホストを`idealis-core`、SMTを`idealis-smt`へ変更した。Go moduleは`github.com/mizchi/idealis/adapters/go`で、package名もidealisにした。rtla、rtla-compiler、rtlcの独立名は維持した。

## 配布とCLI

package consumerを先に新しいartifact名へ切り替え、旧archiveだけでは失敗するRedを保存した。新しいnpm/Cargo/Wasm archiveを作り、別consumerでTS、Rust、Goを検査した。CLIはインストール済みの`.bin/idealis`を直接起動し、適合、違反、不正な操作に対する終了コード0/1/2を検査した。

README、利用ガイド、公開metadata、workflow、ビルド・ベンチマークの現在のパスも更新した。[移行ガイド](../../docs/migration.md)にimport、crate、CLI、remoteの対応表を記載した。registryへはpublishしていない。

## 保存データの互換性

format IDの`modelt.*`とWasm ABI v1の`modelt_*` / `smt_*`は維持した。過去のplan、iteration、runとvendor/fixture原本は編集していない。新しい名前のloaderと実装で、既存の有限モデル、トレース、backend request、Goのfixtureを検査した。

SMTは保存した251リクエストの応答がnative/Wasmとも一致し、215件のSAT値をZ3で再検査した。既知入力の回帰検査であり、新しい独立評価や速度改善とは数えない。

## 検証とGitHub

check/browser、portable、Go、cross-language、async、safety、SMT、runtime、usable、既存evaluate/replay、package、mutationを検査した。[Red、各検証ログ、consumer結果とdigest](../runs/0033-rename-idealis/result.json)を保存した。

[最終archiveのconsumer検査](../runs/0033-rename-final/result.json)には、移行ガイドを含む最終npm archiveのdigestも保存した。

ローカル検証後にGitHub repositoryを`mizchi/idealis`へrenameし、originを新URLへ切り替えた。ghqのcheckoutも新しい名前へ移した。変更はPRとして公開し、mergeは行っていない。
