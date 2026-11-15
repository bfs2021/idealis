# 0027: Apalacheとrtlcの方式・保証・費用を比較する

Apalache v0.63.0と現在のrtlcを比較する。rtlcはIRを使う有限状態の列挙、ApalacheはSMTによるbounded model checkingと帰納的不変条件の検査であり、単一の速度倍率へまとめない。機能と配布要件は公式docとrelease/sourceを参照し、機械確認した結果と分ける。

## 先に固定する実測条件

共通のTLA+ source/configを使う。Apalache用のtype annotationはcommentに置き、rtlc側でも同じsourceを解析する。ASSUME、WF/SF、Intの無限領域、型の異なる値の比較、precedenceが版で異なるSUBSET/DOMAINはpilotへ混ぜない。rtlcへは既存TS frontendでIR化して渡す。

ケースは次の6条件にする。

- 16状態ringの正常版、深さ4。
- 同じ正常版、深さ15。
- 同じringでx=8に違反する版、深さ4。
- 同じ違反版、深さ8。
- 8要素の0/1関数を全初期値とするモデル、正常版、深さ4。
- 同じ関数でf[1]=0を要求する故障版、深さ4。

各条件で1 warmupと3測定を行う。順序は各反復で入れ替える。process起動からJSON解析/結果判定までのwall timeを記録し、rtlcはsource解析からIR生成までを含む時間と、事前IRを使う時間を分ける。Apalacheはfresh JVMでparse/typecheck/SMTを含め、常駐serverの性能と比較したと解釈しない。計測中はbuild/testを並走しない。rtlcは逐次、Apalacheはdefault incremental/Z3。heap、toolchain、binary/source hash、コマンドとraw logを記録する。wall timeoutは追加しない。

## 保証と対照

Apalacheの反例なしは指定長以下の範囲であり、rtlcのcompleteやboundedと同じ判定名へ潰さない。ringの深さ4で両者の反例なし、深さ8で故障の反例、rtlcの正常版の閉じた到達グラフを確認する。対応外、solver/環境失敗、unknownはerrorとして残す。生成された反例は初期状態・遷移・最終違反を元のモデルで独立に確認し、SUT実装の未知バグ発見には数えない。

正常ringについて、Apalacheのlength=0による初期条件と、Initを不変条件へ置いたlength=1による保存条件も別に実行する。固定された16状態のモデルに対する帰納性であり、任意のパラメータに一般化しない。性能表へ追加サンプルとして混ぜない。

## 記録

新規runへsource/config/IR、各sample、反例、raw log、digest、期待結果と一致したかを保存する。既存runは上書きしない。途中の失敗も残し、新規IDで再試行する。比較docsには機能表、今回のpilotの限界、modeltでのbackend選択と次の大規模比較条件を記す。just check/browserとevaluate/replayを確認し、既存runtimeのコードを変更しない。
