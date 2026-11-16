# 0040への追記: 空Boolean applicationの扱い

開発中のnode回帰テストで、Z3 5.4.1の`Solver.fromString`が`(assert (and))`をarguments missingとして拒否した。native Z3 5.4.1でも`(assert (and))`と`(assert (or))`が同じerrorになる。後続check-satのSATは、拒否されたassertionを検証した結果ではない。

0040に書いた「空and/orはZ3互換なtrue/falseへ変換する」という想定を取り消す。SMT-LIB parserの既存の拒否動作を保ち、元のRustテストを復元し、native/Wasm/browserでも拒否することを検査する。失敗ログは`.artifacts/smt-expansion/node-first.log`に保存した。

予約した開発seed 0x40a1、確認seed 0x40c3はこの時点では未実行。残りの式・context API・確認条件は0040を維持する。新しいsource fingerprintにはこの追記も含める。
