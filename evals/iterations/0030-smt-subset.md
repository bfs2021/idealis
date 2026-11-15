# 0030: 整数の差分制約をRustで解く

[事前計画](../plans/0030-smt-subset.md)に従い、[実験用のRustソルバー](../../docs/smt-subset.md)を実装した。任意精度整数とBoolean式を受け取り、差分制約の負閉路検査からSATモデルまたはUNSATを返す。未対応はerror、work予算切れはunknownとして区別する。

## 正しさと実装の故障再現

| run | 入力 | SAT | UNSAT | 拒否対照 |
| --- | ---: | ---: | ---: | ---: |
| [開発最終](../runs/0030-smt-development-final/result.json) | 74 | 64 | 10 | 12 |
| [確認](../runs/0030-smt-confirmation/result.json) | 170 | 159 | 11 | 12 |

開発はseed `0x30a1`の64件、確認は`0x30c3`の160件。それぞれ固定10件を加えた。生成入力は独立したTS evaluatorで3整数の`-2..2`と2真偽値を列挙し、期待する充足性を決める。Rust native/WasmとZ3 5.4.1の結果が一致し、Rustが返した全SATモデルを元の式に固定してZ3でも検査した。

任意精度、厳密比較の負閉路、parallel let、Boolean ite、引用識別子、条件付きの定数を固定入力で確認した。prepare/check/release、解放済みhandle、work 0のunknownも検査した。開発最終と確認はsource fingerprintとbinary hashが同じ。

TTLの式から生成した具体値を、既存の故障版と正常版の実装へ渡した。境界を誤る故障版はmismatch、正常版はmatchedだった。既知の人工故障の再現であり、未知バグの発見数には含めない。

## Redから見つかった差分

Rust APIが存在しないRedから実装を開始した。[初期の開発run](../runs/0030-smt-development-initial/result.json)は64生成入力と10拒否対照で、その後の最終条件とは区別する。

追加の固定入力で、Rustが`(and)`と`(or)`を空の単位元として受け入れ、Z3は拒否する差を検出した。拒否するRustテストを追加してRedを保存し、parserのarity検査を修正した。最終runでは両方を拒否対照に加え、単項のand/orは受け付ける。

## 初回の速度測定と解釈

[初回の4経路比較](../runs/0030-smt-benchmark/result.json)は17入力、680計測sample、136 warmup、12 cold sample。native/WasmともZ3 5.4.1を使い、SATモデル取得とtransportを含めた。小さいparse+solveはRustが速い一方、Z3 nativeのprepared Boolean UNSATはRustより速く、Boolean 12のUNSATでRustは予算切れになった。

この比較のparse+solveではZ3 CLIをresetし、Z3 WasmのSolverを毎回生成している。TTLでnative約146倍、Wasm約315倍という差が出たが、solver kernelやRust言語の倍率とは扱わない。[0031の追加計画](../plans/0031-smt-reuse.md)を先に記録し、Z3のpush/popによる再利用も比較することにした。最終の採否は[0031の結果](0031-smt-reuse.md)に記録する。

## Criterionと実ブラウザ

[Criterion run](../runs/0030-smt-criterion/result.json)はparse+solveとpreparedをchain 4/32/128で測った。warmup 1秒、measurement 2秒、20 sample。chain 128のparse+solveは366.90 µs、95%区間364.76-368.38 µs。preparedは187.62 µs、区間184.85-190.52 µsだった。transportとZ3の比較を含まないRust内の測定で、差は解析を省いた効果として扱う。

Rust 5テスト、Criterion 6 smoke、NodeのZ3差分、実Chromiumの検査を通した。共通77テスト、既存browser、runtime-build/test、evaluate/replayも通した。[検証ログと失敗ログ](../runs/0031-smt-verification/result.json)を保存した。mutation対象とpublic APIは変更していない。
