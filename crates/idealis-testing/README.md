# idealis-testing

有限状態機械からの操作列生成、削除による縮小、保存caseの再生、差分比較を行うRustライブラリ。serde/serde_jsonだけに依存し、同じ実装をnativeとホストimport不要のWasmで使う。SUTのI/Oやテストランナーはホストが担当する。alpha版で、registryには未公開。

## セッションを駆動する

`Session::property(Plan)`を作り、`effect()`を読む。Evaluateはcorrelation ID、phase、caseを持つ。ホストが新しいSUTを初期化してcase.actionsを実行し、`resume(id, verdict)`へ評価を返す。Doneが最終結果。Rustの呼び出し例は[examples/counter.rs](examples/counter.rs)を参照。

Verdictはmatched、signature付きmismatch、invalid、reason付きunknown、message付きerror。mismatchを検出すると同じsignatureを保つ削除候補を検査し、最後に再現を確認する。unknownの候補がある場合、縮小をminimalと判定しない。異なるsignatureや非再現を成功へ変換しない。

結果はpassed、failed、exhausted、inconclusive、error、not-failing。passedは指定した数の有効な正常caseを実行した意味で、モデルの完全探索ではない。invalidだけならexhausted、work不足ならinconclusive。縮小のminimalは、生成器が受理する列の中で単一削除しても同じ失敗を保持できない意味。予算で打ち切った縮小はboundedになる。scalar値の縮小は含めない。

`Session::shrink(generator, case, max_checks, max_work)`は既知のcaseを縮小する。`Session::replay(case, expected_signature)`は保存したactionを使い、乱数から再生成しない。expected_signatureを指定した再生が失敗を再現しなければerrorになる。unknownや実行errorは別に保持する。

## 生成器と保存形式

Generatorはid/version、初期状態、各状態のaction/target、minLength/maxLengthを持つ。最初に目標長、初期状態、遷移をxorshift32-v1で選ぶ。seedは1-2^32-1。終端へ到達するとそこで止め、minLengthに届かない生成はskipする。縮小候補は同じ有限状態機械と長さの条件を満たす必要がある。汎用Proptestやfast-checkのStrategyとseedの互換性は提供しない。

Caseはformat=idealis.test-case、version=1、rng、generatorId/generatorVersion、seed/index、全actionsを持つ。モデルやDriverの版は評価側のmanifestへ固定する。JSONでは配列の順序を保持し、objectのfield順は比較に影響しない。数値はsafe integerに限り、任意精度整数・集合等はホストで既存のPortableValueへ正規化してから渡す。

上限は1024状態、10,000遷移、128 actions、action当たり8 KiB、JSON再帰32段、case/request当たり1 MiB。1セッションは最大256正常case、4096 attempts、512縮小checks、16 MiBのtrial記録を扱う。超える入力はprotocol/resource error。workは生成の各段階と候補の受理検査を数え、JSON復号・plan検査・SUT実行の時間を含めない。SUT評価回数はattempts/checksで制限する。

## 差分比較

`compare(Observation::Allowed { values }, targets)`は、referenceが許可する値集合と各実装の観測を比較する。対象を多数決でオラクルへしない。空の対象はerror、空の許可値集合はinvalid。invalid/unknown/errorはmatchedへ変換せず、対象ごとのoutcomeも残す。全体statusはerror、mismatch、unknown、invalid、matchedの優先順で決めるため、全体statusだけで他対象の判定が完了したとは扱わない。

## nativeとWasm

binaryはversion=1のJSONLを扱う。requestのkindはproperty、shrink、replay、resume、cancel、compare。property/shrink/replayは同時に1セッションだけ実行し、cancelで解放する。compareは進行中のセッションとは独立。無効なrequestやfeedback IDは進行中の要求を消費しない。

Wasm ABI v1はtesting_abi_version、testing_input_resize、testing_dispatch、testing_output_ptr、testing_output_len。ホストは同じJSONを渡す。並列に実行する場合はworkerごとに別runtimeを使う。

[Proptestの状態機械テスト](https://proptest-rs.github.io/proptest/proptest/state-machine.html)と既存TSの縮小器を参照した。外部solver、並行スケジュール探索、高階Strategy、scalar縮小はこのcrateへ組み込んでいない。
