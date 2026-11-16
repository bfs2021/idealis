# Apalache fold の TLC 対照

2026-11-16 に TLC v1.7.4、revision `5a47802b5c391f59ecdd44117981f4ff8c0656ba` で採取したログ。jar の SHA-256 は `936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88`。Apalache 0.56.1 の `tla2sany/StandardModules/Apalache.tla` の SHA-256 は `a9115b404e72c58fd085f81bc59cc918f8adac49feabfb57f8e285bc7ab5cfac`。採取前に両方を検証した。module 原本はリポジトリに含めず、隔離した scratch の各ケースディレクトリにだけ配置する。

入力は `evals/tlc-differential/corpus.ts` の `foldCases` / `foldOrderControls` / `foldRejections` の37件。すべてに `goldenFrontendParity` を宣言し、受理時の TS / Rust frontend の IR、拒否時の両 frontend の拒否、および native / Wasm を比較する。source と config を各ケースの `Differential.tla` / `Differential.cfg` にそのまま保存する。`TLC_JAR` は pinned jar の絶対パス、`TLC_TMP` はケースごとに分離した一時ディレクトリ。JVM は1つずつ起動し、この枝に残す全ログを再採取した。

```sh
java -Xmx256m -Djava.io.tmpdir="$TLC_TMP" -cp "$TLC_JAR" tlc2.TLC -workers 1 -nowarning Differential.tla
```

stdout と stderr を同じストリームへ保存し、次の置換だけを順に適用して `.log` に保存した。`temporary_directory` と `case_directory` は取得時に使った絶対パスで、パスの置換は Python の `str.replace`、残りは `re.sub` による。

| パターン | 置換後 |
| --- | --- |
| `str(temporary_directory)` | `<tlc-tmp>` |
| `str(case_directory)` | `<capture>` |
| `\[pid: \d+\]` | `[pid: <pid>]` |
| `\bon \d+ cores\b` | `on <cores> cores` |
| `\([^\n]*, (MSBDiskFPSet, DiskStateQueue)\)` | `(<host>, \1)` |
| `\b\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\b` | `<time>` |

ホスト説明の OS・architecture・JVM は置換し、FP set と queue の名前は残した。TLC のメッセージ、ソース位置、状態数・生成数、seed、fingerprint の行は変更していない。`expected.json` に source / config / 置換後 log の SHA-256、終了コード、判定、正常終了時の状態数・生成数、エラー時の文言と位置を記録する。
値は TLC の `x = ...` から PortableValue へ転記し、idealis の出力を期待値に使わない。値の対照は `Prop == x # x` の初期 invariant violation で取得する。Next の対照は2状態・3生成の完全探索。fixture の `unsupported` は TLC の失敗を表さず、TLC が値を返す入力を idealis の対応 fragment で拒否することを表す。

| 対照 | ケース | TLC / idealis |
| --- | --- | --- |
| 整数・Boolean | `fold-set-integers`, `fold-set-booleans` | 数値順、FALSE / TRUE の順が一致 |
| tuple / sequence・集合 | `fold-set-tuples`, `fold-set-sets`, `fold-set-recursive-values` | 要素数、その後に再帰的な辞書順が一致 |
| sequence の要素種 | `fold-sequence-values`, `fold-sequence-functions` | 文字列・record・関数・tuple・集合・Boolean・整数を先頭から処理 |
| 引数順・空・scope | `fold-sequence-left`, `fold-empty-set`, `fold-empty-sequence`, `fold-let-operator`, `fold-nested`, `fold-bound-capture`, `fold-next-action` | 累積値が第1引数で、列は先頭から処理する。空では body を評価せず initial を返す |
| 通常の名前解決 | `fold-own-ApaFoldSet`, `fold-own-ApaFoldSeqLeft` | EXTENDS Apalache のない同名の通常 operator |
| 未対応の集合要素 | `fold-unsupported-{string,record,function}-{direct,tuple,set}` の9件 | TLC は値を返す。idealis は全件を同じ unsupported エラーで拒否 |
| 混合型 | `fold-set-mixed-kinds` | 両方とも評価エラー |
| fragment 制限 | `fold-reject-lambda`, `fold-reject-operator-parameter`, `fold-reject-recursive`, `fold-reject-other-apalache` | TLC は値を返す。frontend は拒否 |
| 不正な形式 | 残り7件の `fold-reject-*` | TLC の input error と frontend 拒否。正常の意味論一致には数えない |

集合 fold の対象は、整数・Boolean・tuple/sequence・集合と、それらの再帰的な組み合わせ。集合の要素が文字列・record・非 sequence 関数をどの深さでも含めば、`Unsupported TLA input: set fold order for strings, records or functions` とする。既存の値表現では domain が `1..n` の関数は sequence と同じ値であり、sequence として扱う。順序に依存しない sequence fold は要素種を制限しない。model value は既存の未対応を維持する。

順序は pinned TLC の [TupleValue](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/value/impl/TupleValue.java) と [SetEnumValue](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/value/impl/SetEnumValue.java) の比較と照合した。関数比較の変更、整数 function domain 幅の制限、文字列や record field の順序調査は後続の value-order 作業へ分離した。この枝の既存値比較は変更していない。
