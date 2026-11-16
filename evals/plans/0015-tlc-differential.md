# TLC を直接オラクルにする差分検査

既存 scalar32入力、複合値・temporal46入力を現在版で再実行する。その後、期待判定を Rust から作らず、同じ TLA+ source / config を TLC・Rust native・Wasm に渡す差分ハーネスを追加する。正常・違反・empty・model error と、compiler の未対応・予算切れ・実行基盤の失敗を分ける。

境界値は空区間と型の異なる所属、SubSeq の範囲と空列、集合の部分集合・比較の短絡、空 domain / domain 不一致時の関数・record の所属、Seq を含む range、EXCEPT の未存在 path、文字列の Len / Tail / 連結 / SubSeq を固定する。文字列は portable value の Unicode scalar 範囲を前提にし、surrogate pair を分割する結果は今回の対応範囲に含めない。

状態数2〜4の有限遷移モデルを seed 0x15A1 で32個生成する。初期状態の個数、遷移、state property / eventually / always-eventually / eventually-always / leads-to と論理結合、WF / SF を変える。開発ケースが一致した後、未使用の確認 seed 0x15B2 の32個を実行する。確認結果を見た修正が必要なら、その run を保ち、新しい確認条件を先に登録する。

native と Wasm は結果・trace・work を完全比較する。TLC とは API 名の差を正規化した判定、complete / empty の状態数・生成数を比較する。有限な小グラフでは liveness 判定時の数も照合する。反例は TLC に強制再生し、全 state の到達・元の遷移・fairness と違反を確認する。有限反例の長さも照合する。TLC の構文・設定・JVM エラーをモデル評価エラーとして合格させない。明示的なエラー対照を残し、error 一致数を complete 数へ加えない。

差分は打ち切らず入力・両結果・生ログ・分類を保存し、mismatch / incomplete / infrastructure-error を結果に残す。TLC と source / Wasm / native binary の digest、revision、seed と toolchain を固定する。最初の mismatch は Red として保存し、TLC ソースと実行で原因を確認してから修正する。修正後も旧 run を変更しない。CI は新しい境界ケースと native / Wasm / tla-ts の回帰検査を行い、TLC の手動 run / replay も justfile に登録する。全 TLA+ 互換や未知の実システムのバグ検出とは扱わない。

開発 run の補足: TLC v1.7.4 (`5a47802b5c391f59ecdd44117981f4ff8c0656ba`) の SANY は非BMP文字を含む2つの入力を字句エラーで拒否した。原入力は残し、同じ2入力を明示的な拒否対照にする。Rust frontend も拒否することを検査し、モデルの一致件数には加えない。初期 invariant violation では集計数を出さないため、出力のある数だけ照合し、欠測を0と解釈しない。TLC master と v1.7.4 は String の Head / Append / toTuple の扱いが違うため、jar と同じ tag のソースで修正する。確認 seed は未実行のまま維持する。

初回の確認 seed 0x15B2 は32件一致した。保存結果の replay では、未実行の native 欄を `undefined` 付きで返すと JSON 保存時に欄が消え、比較が失敗する問題を2件の抜粋で再現した。未実行欄を省略する修正だけを行う。モデル検査器・入力の意味・binary は変更しない。元の確認結果を残し、追加確認 seed 0x15C3 をここで予約する。追加確認を従来32件の新しい独立実行として水増ししない。

並列回帰実行中の再実行で、標準モジュール解析の NullPointerException が1件発生した。同一入力は先行 run で型エラーを返していた。v1.7.4 の `SimpleFilenameToStream.read` は `java.io.tmpdir` 内の固定ファイル名へ書き、終了時に削除する。全 TLC 呼び出しに専用の一時ディレクトリを指定し、JVM の解析例外を input error とも model error とも区別する。失敗 run は保持する。検査器の binary・モデル・確認 seed 0x15C3 は変更せず、未実行の追加確認を続ける。

CASE の追加回帰条件: OTHER のない式と action で、最初の真の arm を選ぶこと、全条件が偽なら到達時に評価エラーとソース位置を返すこと、未評価の分岐ではエラーにしないことを固定する。空の Init の invariant を事前評価しないこと、OTHER 付き action、後続 guard の短絡、未評価の action 分岐も含む小さい8入力を境界 corpus に追加し、同じ pinned TLC v1.7.4 (`5a47802`) の環境情報だけを置換したログと source / config / log / jar の SHA-256 を `evals/fixtures/tlc-case/` に保存する。TS / Rust frontend の IR と native / Wasm の結果を比較し、IR の location 検証も回帰で確認する。確認 seed や保存 run は変更せず、未知故障発見や全 TLA+ 互換の主張は追加しない。

Apalache fold の core 回帰条件: root の直接の `EXTENDS Apalache` と、2つの零 arity 引数を持つ可視の非再帰 operator 名に限定する。累積値が第1引数、要素が第2引数。sequence は先頭から処理し、要素種を制限しない。集合は既存順序が pinned TLC と一致する整数・Boolean・tuple/sequence・集合と、それらの再帰的な組み合わせだけを受理する。文字列・record・非 sequence 関数を再帰的に含む集合は単一の unsupported エラーとする。既存の function 比較は変更しない。関数順序の整合、文字列/field の intern ID、整数 domain 幅の調査ログと期待値は後続の value-order 作業へ分離する。

採取前に core corpus を固定する。既存の空集合・空列、LET、外側の bound capture、入れ子、Next の対照を残し、再帰的な受理と各未対応種の singleton・tuple 内・集合内の拒否、sequence の関数要素、EXTENDS なしの通常名前解決を追加する。意味論の期待値は TLC ログからのみ記録する。全 fold 入力に `goldenFrontendParity` を宣言し、IR または拒否の両 frontend 一致と native/Wasm の結果を比較する。重複した compiler/browser の意味論テストは削減し、IR 検証・scope・予算・frontend 制限の固有リスクを別途確認する。

同じ pinned TLC v1.7.4 (`5a47802`) と Apalache 0.56.1 module（SHA-256 `a9115b404e72c58fd085f81bc59cc918f8adac49feabfb57f8e285bc7ab5cfac`）で、この枝に残す全 fold ログを逐次再採取する。環境情報は CASE fixture と同じ置換規則だけで除去し、source/config/log の全 digest を更新する。確認 seed、保存 run、ledger、探索上限は変更しない。新しい採用・性能・未知故障発見の主張は含めない。

Value-order regression conditions (fixed before capture): compare functions by domain size, then each key/value interleaved. Audit set and function-key normalization, finite-domain enumeration, quantifier short-circuiting, first counterexamples, membership and equality errors. Compare complete responses for the existing corpus before and after the change; stop if an existing golden or saved-run result changes. TLA+ CHOOSE expressions remain unsupported in both frontends; use initial-state selection and quantifier short-circuiting as the observable controls. Portable JSON normalization keeps its independent ordering.

Accept integer-key functions recursively in set folds. Reject when the maximum minus minimum of all integer function keys inside the members exceeds i32::MAX. Freeze interval/explicit-domain controls at and beyond the bound, recursive containers, wide primitive integers, and wide integer function/tuple values that are not keys. Write the minimum integer as -2147483647 - 1 because TLC rejects the positive literal 2147483648. Keep the existing rejection for other function keys. Reject strings and records separately because the IR lacks TLC intern IDs. Capture reversed string appearance, record fields, strings in tuples/sets/integer-key functions, and records inside functions. Sequence folds continue accepting every element kind.

Every new input declares goldenFrontendParity and uses the shared TLC differential regression as its semantic owner. Verify pinned jar/module hashes, capture new logs sequentially, and apply only the CASE replacement rules. Record every new regression Red and Green, verifying temporary edits are restored byte for byte and by digest. Run compiler-eval into scratch with ledger appends prevented; compare against the parent 221 matched / 2 rejected. Leave saved runs, ledger and confirmation seeds unchanged.

Function-equality regression conditions (fixed before capture): compare each normalized key and its value before advancing to the next entry, stopping at the first unequal key/value or evaluation error. Add direct Init equality and ApaFoldSet body equality for functions with domains {0, 2} and {0, 3}, whose values at key 0 are TRUE and 0. Both inputs declare goldenFrontendParity; take the expected evaluation error from pinned TLC logs, then demonstrate Red before changing Arena::equal and Green afterward. Check the interval/interval and both interval/explicit equals branches against the expanded integer-key representation, plus RecordValue, TupleValue and SetEnumValue; leave set equality unchanged. Preserve the bounded iterative fold validation, integer-key span bound, rejection messages and traversal order. Keep saved runs, ledger and confirmation seeds unchanged.
