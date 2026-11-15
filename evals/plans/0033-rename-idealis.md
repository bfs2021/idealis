# 0033: プロジェクト名と配布名をidealisへ変える

プロジェクトとCLIをidealis、npmを@ mizchi/idealis（実際のpackage名では空白なし）、Rust hostをidealis-core、SMT crateをidealis-smtにする。GitHub repositoryをmizchi/idealisへrenameし、originとghqのcheckout名も合わせる。rtla、rtla-compiler、rtlcの独立名は維持する。

保存済みのformat ID、Wasm ABI v1のexport名、過去のplan/iteration/run、vendor/fixture原本は変えない。現在のimport、crate/module、CLI、artifact、workflow、利用ガイドを更新する。未公開のregistryへのpublishは行わない。

単独package consumerを先に新しいpackage名とCLI名へ切り替え、旧archiveでは失敗するRedを保存する。新しいnpm/Cargo/Wasm archiveからのTS/Rust/Go単独利用、CLIのmatched/mismatch/invalidに対応する終了コードを検査する。保存形式は旧modeltのまま受け入れる。

check/browser、portable-build/test、Go、cross-language、async、safety、SMT、runtime-build/test、usable、evaluate/replay、packageを検査する。共有テストを変更した場合はmutationも実行する。既知入力の回帰検査を新しい独立評価や速度改善とは数えない。GitHub renameはローカル検証後に行い、過去runは編集しない。mergeは行わない。
