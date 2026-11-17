# 0050: アクション単位のcoverage

## 範囲と契約

0047の順番に従い、coverageの最初の段階としてInit/NextのIR branch単位の診断を追加する。式単位のcoverage、TLCの全cost model、SYMMETRY/VIEWは別の未対応項目として残す。

通常のSafetyResultとcheckの契約を維持する。Rustのcheck_with_coverageと、native/Wasm共通のcheck-safety-coverage要求を追加する。TSからcheckSafetyCoverageで呼ぶ。結果はresultとcoverageを分ける。coverageはinitial/nextの配列でIRのbranch順と名前を保持し、同名の行を統合しない。

各行はattempts、enabled、generated、distinct、workを持つ。attemptsはbranchを評価し始めた回数、enabledは少なくとも1つの完成した候補を生成したattempt数。generatedは重複・constraint除外を含む完成した候補数、distinctは保存した新規state数。workはそのbranchの評価と受理時のconstraint/property検査を含む差分で、TLCの式評価costとは区別する。unattributedWorkには事前検査とtemporal検査等の差分を残す。行のgenerated/distinct/workの和は全体と整合する。

coverageは逐次検査に限定する。並列coverage、source location別の集計、式のsource mapは未対応。既存APIのoff経路はconst genericで計測を除外する。on/offのstatus・state/generated/work・予算切れ・反例・エラーを一致させる。評価エラー時に成功したpartial reportを返さない。

## 意味論と固定対照

TLC revision 5a47802のWorker.addElement、ModelChecker、coverage.ActionWrapperとcoverageテストを読む。TLCの-coverage 0でgenerated/distinctを比較する。IR分解とTLCのaction粒度が異なる場合は分け、TLCのtimeやcostと同一とはしない。

有限counter、常に無効なbranch、重複するbranch、複数Init、非決定的な候補、空のInit、state/action constraint、invariant/deadlock/action/temporal違反、fairnessと小さいwork/state/depth/temporal予算を固定する。正常/故障の対照と反例再生を使い、未完了をcoverage達成と解釈しない。通常APIとの一致は既存corpusにも適用する。

## 費用と運用

実装前に既存rtlc Criterion14本をbeforeとして測り、off経路のafterと比較する。20 sample、warmup 0.5 s、measurement 1 s。on/offの追加計測は同じprepared IRで行う。計測中はbuild/test/TLC/profileを並走させない。専用機ではないため、後退と揺れを保存し、全体の高速化を主張しない。

plan → Red → Green → Refactoringで進める。native/Wasm、Chromium、strictなwire契約、fmt/clippy/test、just check/browser、safety-test、runtime-build/test、evaluate/replay、mutation、package consumerを確認する。SUPPORT.mdでaction coverageと式coverageを分け、Jevを同期しstaged/オフラインで鮮度を検査する。raw runは上書きしない。
