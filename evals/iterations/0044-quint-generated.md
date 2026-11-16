# 0044: Quint生成TLA+の実行

[計画](../plans/0044-quint-generated.md)に従い、PR #52〜#54をmain `9e1e20d`へrebaseした。Quint 0.33.1とApalache 0.63.0を実行し、固定した6モデルからTLA+を生成した。生成元・stdoutそのもののTLA+・コマンド・stderr・SHA-256を[fixture](../fixtures/quint-generated/generated/manifest.json)へ保存した。

最初の生成前検査では`item`と`next`がQuintの組み込み名と衝突した。束縛変数だけを変更し、操作・invariant・入力集合は変えていない。`--out`へはJSONが書かれ、TLA+がstdoutへ出る0.33.1の挙動も確認し、生成器はstdoutを保存する方式にした。

最初の実行回帰は12件すべてが`:=`の構文解析で失敗した。parserへTLCの比較と同じ優先順位を追加し、`EXTENDS Apalache`がある場合だけ同モジュールの定義`__x := __e == __x = __e`としてloweringした。生成物の`EXTENDS TLC`と`Variants`も受理し、使用するVariant・VariantTag・VariantGetUnsafeをApalache 0.63.0の関数表現へ合わせた。VariantTagはsingletonの文字列domainに限定し、空・複数key・非文字列keyは拒否する。これは一般のCHOOSEや他のVariants operatorへの対応ではない。

TLCとの最初のpilotは、初期状態のinvariant違反ではTLCが状態数・生成数を出さないことをハーネスが扱えず失敗した。[run](../runs/0044-quint-pilot/error.json)を残し、欠測はnullのまま扱うようにした。次のpilotでは9件まで一致し、matchの反例をTLA+へ直す際に、既存のserializerが入れ子の関数で同じ束縛名を再使用してTLCに拒否された。[run](../runs/0044-quint-pilot-second/error.json)を残し、深さごとに束縛名を分けた。検査を弱めず、生成TLA+は一切書き換えていない。

[3回目のpilot](../runs/0044-quint-pilot-third/result.json)では12件の判定と、TLCが出力する状態数・生成数がnative/Wasmに一致した。counter・Choice・Matchの違反は遷移後、foldの違反は初期状態で検出した。返却された6件の反例を、原本をimportする別モジュールでTLCへ渡し、元の遷移を満たして違反へ到達することを確認した。正常版の状態数/生成数はCounter 4/5、SetFold 1/2、SequenceFold 1/2、Choice 3/10、Match 3/4、NestedFold 1/2。

回帰用の[オラクル](../fixtures/quint-generated/tlc-oracle.json)はTLCログから作り、idealisの出力を期待値へ転用していない。通常の回帰はQuintやJVMを起動せず保存TLA+とTLC判定を使う。独立したfixtureの13テスト、実Chromiumで12入力の検査、再生成・TLC照合・保存run再生のjust recipeを追加した。

既存のVariants import全体を拒否する対照は、今回の限定した対応と矛盾するため更新した。新しい正常受理の回帰に加え、Realsのimport、未対応のVariantFilterとTLCのPrint、foldの展開上限を拒否する対照を残した。

ソース`c59e098`を固定した[最終run](../runs/0044-quint-final/result.json)と[再生run](../runs/0044-quint-replay/result.json)も、同じsourceVersionとbinaryで12件・反例6件が成功した。再生ではnative/Wasmの全結果も保存値と完全一致した。[検証ログ](../runs/0044-quint-verification/manifest.json)にportable-test、check、browser、mutation、evaluate/replay-evaluation、safety-test、quint-testを保存した。既存compiler-evalは237入力・235受理・2拒否を維持した。Chromiumは新しい12入力を含め3テスト、保存fixtureは13テスト、safety回帰は211テストが通過した。

採用するのは、この生成形式の有限fragmentをRust/native/Wasmで実行する経路。Quint言語全体やQuint ConnectのAPI・ITF、任意コードとの適合性、性能は今回の評価対象ではない。結果は人工的な固定モデルの互換性回帰であり、実プロジェクトの未知バグ発見とは数えない。
