# 0046: TLCの有限モデル対応を増やす

rtlcとrtla-compilerの対応表を、構文・設定の受理、実行時の意味論、探索方式、ホスト機能に分ける。model value、state/action constraint、有限CHOOSE、整数のdivision/modulo/power、定数の有限domain上の時間論理量化を最初の対象にする。SYMMETRY、VIEW、checkpoint、標準moduleの残り、simulation/DFID、coverage、ALIASは個別の未対応項目として残し、この段階で対応済みとはしない。

比較元はローカルのtlaplus/tlaplusのModelValue.java、Integers.java、Tool.java、Worker.java、ModelChecker.java、Liveness.java。TLCの同じsource/configをnative/Wasmに渡し、正常・反例・評価エラーを分ける。constraint境界のinvariant、初期状態、deadlock、action property、活性グラフを確認する。CHOOSEの無解はエラー、候補順と未評価の分岐も検査する。整数は負数、ゼロ、i32境界、大きな指数を含める。時間論理量化は空domain、複数要素、tuple、非定数domainの拒否を含める。

TDDで回帰を先に追加する。人工的な正常/故障と境界の差分を互換性の証拠とし、実システムの未知バグ発見や全TLA+互換性とは数えない。保存入力を再実行し、native/Wasm一致、TLCの判定と計数、反例を確認する。既存の意味論に差があれば修正前の失敗も新しいrunへ残す。

変更前後に既存14本のCriterionを各20sample、warmup 0.5秒、measurement 1秒で測り、source digest、環境、入力、raw log、推定区間を保存する。frontendとIPCを除くnative API/kernelの測定であり、JVM全体との倍率にはしない。追加機能は固定source/configのcompileとcheckを分けて測定する。既存機能の後退は隠さず、ばらつきが大きければ同条件で再確認する。新機能を実行できない旧版に対する速度倍率は出さない。

checklistとREADME、Jevの保存比較を同期する。終了前にfmt/clippy、crate回帰、compiler/safety/portable/browser、check、evaluate/replay-evaluationを確認する。旧runを編集しない。
