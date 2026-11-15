# 0034: 静的なAlloy関係モデルをRustで検査する

`crates/ralloy`を独立crateとして追加する。Alloy 6.2.0の言語仕様と実装を参照し、静的なトップレベルsig、set/one/lone/someの二項field、fact、引数なしpred、assert、run/checkを対象にする。関係演算は和・差・交差・積・join・転置・推移閉包・反射推移閉包、集合の等価/包含、量化と論理結合を扱う。

Alloyの通常scopeは上限で、exactlyは個数を固定する。one/lone sigの暗黙scopeと明示scopeの制限、Intの既定4-bitと0-bitを参照実装に合わせる。整数演算は対象外だが、univとidenに含まれるInt atomは保持する。sigの継承・subset、モジュール読込、引数付きpred/function、時間/var、整数演算、3項以上の関係は未対応として拒否する。

型と名前、全宣言とcommandを解析時に検査し、未選択の宣言にある未対応構文も無視しない。有限scope内の構造を列挙する。runのinstance、checkのcounterexample、scope内のno-counterexample、事実を満たす構造がないno-instance、work予算切れunknownを区別する。予算は時間制限ではなく、unknownにはinstanceを付けない。スコープ外の証明やAlloy全体への互換性は主張しない。

先にRustのAPIテストを追加してRedを保存する。graphのcycle、所有関係、fieldの既定one、量化の空集合とdisj、join/transpose/closure、scope・Int・不正型・予算を検査する。グラフ例はAlloyのutil/graphの制約を静的サブセットへ展開したものとして出典を残す。SAT変換の移植ではなく、最初は予算付きの関係列挙器として実装する。

開発seed0x34a1の32件と確認seed0x34c3の64件を固定する。2-3 nodeの独立TS relation evaluatorで期待値を作り、Rust native/Wasmと公式Alloy 6.2.0/SAT4Jを比較する。Intの組込みやone/loneのscope等は固定ケースで比較する。返した構造をAlloyの式として固定し直し、参照Analyzerが受け入れることを確認する。factが充足できないcheckを成功へ変換しない。

生成した2-cycleで、自己loopしか拒否しない故障版とcycleを拒否する正常版を対照にする。既知の人工故障の再現で、未知バグ発見とは数えない。native/Wasmと実Chromium、単独crate consumer、fmt/clippy/test、check/browser、runtime-build/test、evaluate/replay、packageを検査する。入力・source/binary hash・参照版・全応答と失敗ログを新規runへ保存する。既存backendへの統合、registry publish、mergeは行わない。
