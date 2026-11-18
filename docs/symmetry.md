# node関数に限定したSYMMETRY

Rust frontendとrtlcは、同じ種類のnodeを入れ替えても仕様が変わらない場合に、状態を同一視できる。対象は単一のモデル値集合と、その全要素をdomainに持つ関数のInt/Bool値。任意のTLA+仕様の対称性を自動証明する機能ではない。

## 宣言と条件

```tla
---- MODULE Nodes ----
EXTENDS Naturals, TLC
CONSTANT Members
VARIABLE local
Symm == Permutations(Members)
Init == local = [node \in Members |-> 0]
Next == \E node \in Members:
          local' = [local EXCEPT ![node] = IF @ < 2 THEN @ + 1 ELSE 0]
Safe == local \in [Members -> 0..2]
====
```

```text
CONSTANT Members = {a,b,c}
INIT Init
NEXT Next
INVARIANT Safe
SYMMETRY Symm
```

Rust frontendはflat moduleのゼロ引数定義`Symm == Permutations(Members)`だけを受理する。`TLC`のimportと、CONSTANTへ直接割り当てた2〜64個の異なるuntyped model valueが必要。`n_a`のように先頭1文字とunderscoreで型を持つ値は対象外。複数群、任意の置換集合、INSTANCEや追加module bundleは拒否する。通常の式としての`Permutations`は追加していない。既存TS frontendのSYMMETRY対応は追加していない。

状態変数はInt/Boolか、全Membersをdomainに持つ関数に限定する。各関数内の値はIntだけ、またはBoolだけ。複数関数の型が異なる場合も扱う。node参照をpayloadへ格納するleader/pointer、nested function/record/sequence、部分domainは対象外。

## 宣言が要求する前提

SYMMETRY宣言は、指定したnodeの任意の入れ替えがInit、Next、検査するinvariantを保つという利用者の前提。この前提が成立しない仕様に対して、縮約後のcompleteから元の仕様の安全性を結論してはならない。TLCと同様、ここで仕様のequivarianceを証明してはいない。

rtlcは保守的なgateとして、群の個別要素を直接参照するIRを拒否する。群の全要素を含む定数集合としての使用だけを受理する。CHOOSE、fold、sort/select-seq、ENABLED、sequence操作等も対象外。gateを通ることと、宣言した前提の証明は区別する。安全性を判断する際は縮約なしの小さいモデルとの照合も行う。

初期版はinvariantとdeadlockのsafety検査に限定する。temporal/fairness、state/action constraint、initial/action property、coverageとの併用を拒否する。VIEWは別機能で未対応。nativeの複数worker要求はSYMMETRY有効時に逐次へfallbackする。Wasmでも同じ実装を使う。

## 同一視と反例

各nodeについて、すべての関数の値を同じ行へまとめる。その行のmultisetをkeyにする。関数を独立にsortすると、同じnodeに属する値の相関を失うため行わない。globalなInt/Boolと、各変数が関数かscalarかもkeyへ含める。

この状態形状では、全N!置換の列挙を行わずにorbitを区別できる。hash衝突時はkey全体を比較する。TLCと同じfingerprintのbyte列を作る実装ではない。

探索・反例用には、元の親から生成した具体状態を保存する。同一視のためにnode名を書き換えた状態をtraceへ出さない。新しいsafety反例はSYMMETRYを外したNextへ再生して検証する。livenessと最短lassoは保証範囲に含めない。

## IRと計数

`SafetyProgram.symmetry`はoptionalな`{modelValues: string[]}`。Rustでは`rtlc::ir::SafetySymmetry`を使う。設定しないJSONではフィールドを省略する。RustでSafetyProgramをstruct literalから構築する場合は`symmetry: None`を追加する。

states・generated・depthは縮約後の探索の計数で、縮約なしと一致する値ではない。workには群の準備、keyの構築と行のsort用の固定費用も含める。workはCPU命令数や厳密なメモリ制限ではない。maxStates/maxDepth/maxWorkによるboundedをcompleteへ読み替えない。

[対応チェックリスト](../crates/rtlc/SUPPORT.md)、[compiler](../crates/rtla-compiler/SUPPORT.md)、[0060の評価計画](../evals/plans/0060-restricted-symmetry.md)を参照。
