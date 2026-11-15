# 0030: RustのSMT-LIBサブセットで求解費用を測る

Z3の置き換えを決める前に、差分制約へ正規化できるInt/Boolのquantifier-free formulaを実験用Rust crateで解く。SMT-LIBのdeclare-const、0引数declare-fun、assert、任意の単一check-sat、Boolの論理結合、Intの加減算・比較・等価、letを対象にする。set-logicはQF_IDL/QF_LIA。QF_LIA全体への対応ではない。

整数は任意精度。無条件の等式で固定された整数定数を代入し、残る制約をx-y<=cへ正規化する。Boolean分岐を探索し、各枝をBellman-Fordの負閉路検査で判定する。有限範囲の列挙でunsatを推測しない。未対応の式はerror、work予算切れはunknown。型エラー・不正SMT-LIB・未対応commandも合格へ変換しない。

先にRustテストを追加してRedを記録する。TTL境界とamount+other=10/other=3を含む既存用途、負閉路、Bool分岐、letの同時束縛、厳密比較、大きい整数、予算切れと非対応を検査する。固定seed0x30a1の64件を開発用、0x30c3の160件を確認用とし、3整数の-2..2と2真偽値の有限対照を独立したTS evaluatorでも列挙する。Z3とのsat/unsatを比較し、Rustの全sat modelをZ3へ戻して検査する。確認結果を新しい独立実験へ数え直さない。

Rust native/WasmとZ3 native/Wasmを測る。Z3 nativeはWasmと同じ5.4.1 releaseを公式digestで検査する。各問い合わせでparse+solve+model取得する常駐経路を主指標とし、prepared checkとfresh process/初期化を分ける。正しさの対照・モデル検証は計時外。モデル値そのものの一致は要求せず、充足性を検査する。

性能入力はTTL、amount、差分chainのsat/負閉路unsat、Bool選択、schedulerの6種類。chainは4/32/128/512変数、Boolは4/8/12選択、他は固定入力とする。1 warmup後5 sampleを交互順で測り、中央値・全sample・最小/最大・workを保存する。coldは各経路3 fresh process。kernelのCriterionはparse+solve/preparedを別に測る。実行環境を専有していない場合も記録する。

主指標でZ3 nativeに対する1.5倍以上の改善が複数familyにあり、Wasmでも利益が確認できる場合に限定的採用の候補とする。後退、work切れ、非対応を隠さない。RustとZ3ではアルゴリズムも範囲も異なるため、Rust言語自体による速度差へ帰属しない。結果にかかわらず既存Z3 adapterは維持する。

実験はevals内に置き、public APIと配布を変更しない。native/Wasm、実Chromium、cargo fmt/clippy/test、check/browser、runtime-build/test、evaluate/replayを実行する。manifestにsource/binary/version/seed/予算、入力・結果・raw logのdigestを記録し、run IDを上書きしない。
