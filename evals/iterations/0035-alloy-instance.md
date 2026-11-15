# 0035: Alloyモデルと固定した観測構造の照合

`Model::check_instance`と`@mizchi/idealis/alloy`を追加した。実装のsig/fieldを固定して評価し、別の充足する構造を探索し直さない。[事前計画](../plans/0035-alloy-instance.md)と[利用手順](../../docs/alloy.md)を参照。

scope・宣言・fact・propertyを満たせばmatched、property違反はmismatch、scope・宣言・fact違反はinvalid、予算切れはunknownとする。check commandもassertの正方向を評価するため、探索からのcounterexampleを照合するとmismatchになる。未知の宣言・atom、重複、欠落field、壊れたschemaはprotocol error。

## 公式Alloyとの構造固定比較

各モデルへ観測したsig/fieldを固定するfactを追加し、公式Alloy 6.2.0/SAT4JでFeasibleとQueryを実行した。FeasibleがUNSATならinvalid、充足可能ならQueryからpropertyの真偽を判定する。実行エラーをinvalidへ変換しない。

| run                                                 | 固定ケース | 生成ケース | matched | mismatch | invalid | 公式との判定一致 |
| --------------------------------------------------- | ---------: | ---------: | ------: | -------: | ------: | ---------------: |
| [開発](../runs/0035-alloy-development/result.json)  |         10 |         16 |       4 |       12 |      10 |            26/26 |
| [確認](../runs/0035-alloy-confirmation/result.json) |         10 |         32 |       7 |       22 |      13 |            42/42 |

生成ケースは新しいseed0x35a1と0x35c3を使った。固定10件は両runで共通で、68件を独立した入力数として扱わない。TSの独立したグラフ走査、native JSONL、公開Wasm APIの判定が一致した。0/1 workの予算切れも検査した。scopeとfield multiplicity、factの充足不能はいずれも公式側ではUNSATなので、invalidのreasonは別の固定期待値とRustテストで確認した。

両runのsourceVersionは`c747cc5c5e2b19d66826a6d1ec2d321b317005bdb40b342cf19e51af57d1d0d0`。全caseの入力・結果、公式receiptとログ、source/native/Wasm/JARのhashを保存した。開発132ファイル・確認216ファイルのdigestを照合し、同じnative/Wasmで応答を再現した。replayでは公式Analyzerを再実行していない。

## テストランナーと配布

同じ3件の契約をnode:testとVitestで実行した。3ノードの全512グラフを独立したTS循環判定、native、公開Wasmへ照合した。固定10件のscope・multiplicity・Int・fact等も両経路で検査した。実Chromiumではホストimportとcross-origin isolationなしで探索、観測構造の一致・違反・予算切れを確認した。

既存の2-cycle故障対照も新しいcrateで再実行した。自己loopだけを拒否する故障版と循環を拒否する正常版の区別を再現し、公開APIから同じ反例がmismatchになることも確認した。既知の人工故障であり、未知バグの発見には数えない。

npmに`./alloy`と`./ralloy.wasm`を追加した。旧archiveではconsumerがexport未定義で失敗することを先に確認し、新archiveだけを入れた別consumerで探索・固定instanceの照合・予算切れ・拒否と型検査を通した。独立Rust consumerも新APIで反例を再照合する。

Rust fmt/clippy/14 tests、check/browser、runtime-build/test、evaluate/replay、package、既存mutationと回帰テストは通過した。既存mutationのscoreは95.4%で、Alloy専用のmutation強度を測定した結果ではない。[検証run](../runs/0035-alloy-verification/result.json)へログと配布物のhashを保存する。

## 採用範囲

関係スナップショットの適合性検査をRust/native/Wasm/TSから使うAPIとして採用する。任意の実装を自動的にモデル化する機能や、全実装入力・時間的な振る舞いの証明は含めない。汎用backend契約、CLI、探索の高速化は別の対象として残す。
