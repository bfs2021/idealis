# 0040: SMTの定数倍・整数iteとcontext API

独立crate `idealis-smt`に、定数倍、整数ite、assertion scopeと一時Bool assumptionsを追加した。一般LIAや他の理論、学習状態の再利用は未対応。[チェックリスト](../../docs/smt-checklist.md)で残りを追跡する。

## 条件と結果

[plan](../plans/0040-smt-expansion.md)を実装前に固定した。Z3のSMT-LIB parserが空and/orを拒否したため、予約seedの実行前に[追記](../plans/0040a-smt-expansion-adjustment.md)し、既存の拒否動作を維持した。最初のRust Redは未実装のSolver/Assumptionへの参照で失敗した。

旧版はplanだけを追加した`95732ef`のnative/Wasmを保存した。新版は同じsource fingerprintとbinaryで開発・確認を実行した。Z3 5.4.1が独立オラクル。SATの返却値を元の式へ代入してZ3へ再照合した。UNSAT、unknown、errorを区別した。

| 評価          |  式 | 旧版で受理 | 新版で判定一致 | contextの照合 | SAT / UNSATの総数 |
| ------------- | --: | ---------: | -------------: | ------------: | ----------------: |
| 開発 `0x40a1` |  76 |          1 |             76 |            21 |           60 / 37 |
| 確認 `0x40c3` | 140 |          1 |            140 |            21 |           84 / 77 |

式は各runに手書き12件と生成64/128件を含む。旧版が受理した1件は既存IDLの正常対照。残りは旧版が拒否した。両版ともnative/Wasmの応答が一致した。新機能を狙った人工入力の結果なので、実用入力全体の対応率とは解釈しない。

contextの期待assertion stackはTS側で別に保持した。追加・入れ子scope・矛盾・pop後の定数代入の復元・一時assumptions・別handleの独立性を検査した。失敗したassert/pop/pushの後も元の問い合わせが通る。256件の上限、解放済みhandle、work 0のunknownと空モデルも検査した。11件の不正/未対応入力はerrorを維持した。

正負・0・任意精度の係数と厳密比較、負のboundのfloor、割り切れない等式を含む。一般LIA `2*x + 3*y <= 7`、非線形乗算、混在sort、256選択肢を超える整数iteは拒否する。未対応branchを含む式を、到達不能と推測して受理する処理はない。

## 保存と再生

- [開発run](../runs/0040-smt-development/result.json)と[確認run](../runs/0040-smt-confirmation/result.json)に入力、問い合わせ、意図したstack、全応答、Z3モデル、source/lock/binaryのhashを保存した。
- source fingerprintは`f553a4467a6021a07b4d085aefef110ee50be58828567f79afa389dc773dc40d`。READMEと結果文書は除外する。実装・評価器・plan・task・CI・consumerのスクリプトは含める。
- 両runを再実行し、native/Wasmの応答・判定・期待stackが保存結果と一致した。Z3は別のモデルを選び得るため、モデル文字列の同一性を要求せず再検証する。
- [検証記録](../runs/0040-smt-verification/result.json)にRed、Z3で判明した想定違い、整形前の失敗、最終チェックのログを保存した。配布の再検査では容量不足が起き、今回作成したconsumerとpackage workspaceの再生成可能なtargetだけを削除して再実行した。

```sh
just smt-expansion-test
just smt-browser
just smt-expansion-eval development <new-run> <old-native-binary>
just smt-expansion-eval confirmation <new-confirmation> <old-native-binary> <development-run>
just smt-expansion-replay <saved-run>
```

baselineのWasmはold-native-binaryと同じディレクトリの`idealis_smt.wasm`を使う。保存runは上書きせず、結果を見たseedは次の独立した確認に再利用しない。

## 採用する範囲

定数倍はscaled differenceへ落ちる場合に採用する。整数iteはguard付き選択肢へ変換し、演算・比較の展開を256に制限する。RustのSolverとJSON context APIは繰り返し問い合わせに使える。宣言は初期sourceに固定し、assert/popは再解析する。変更なしのcheckは解析結果だけを再利用する。

Rust tests・clippy・fmt、node:test/Vitest、旧SMT差分、実Chromium、check/browser、runtime-build/test、evaluate/replayと単独package consumerを検査した。Rust consumerはcrate archiveだけを展開して新APIを呼び、他のidealis crateやZ3への依存がないことを確認した。

速度比較はしていない。SMT-LIB scriptのpush/pop/check-sat-assuming、一般LIA、CDCL、理論伝播、学習状態の再利用、公開backendとCLIへの接続は今後の対象。
