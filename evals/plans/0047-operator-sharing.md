# 0047: operator定義を共有するsafety IR v2

Issue #60の仮説は、非再帰operatorの定義表とlexical frame付きの遅延引数により、call siteごとの本体・引数の複製とbinder slotの混線を除ける、というもの。共有は値のmemoizationではない。本段階は事前計画だけを追加し、実装、TLCログ採取、評価を行わない。結果を見る前の計画確定は後続のorchestratorが担当し、この作業ではコミットしない。

## 変更と互換性

`modelt.safety-program` v2に安定した定義ID、parameter数、capture数、body kind、local scopeを持つ定義表とcall / parameter / capture参照を加える。IDはlexicalな定義順、captureは束縛の宣言順で決め、必要なphase / membership specializationも両frontendで同じ順に固定する。到達する各定義・specializationは一度だけlower・検証し、call siteではarity、capture、phase適合を検証する。再帰は非対応のまま、定義graphのself / mutual cycleをcompile時とIR検証時に拒否する。

選択肢の仮称は全経路で`irVersion: 1 | 2`（Rustの型でも同名フィールド）。`compileTlaSafety`、Rust source API、`compileTlaSource` / `checkTlaSource`、native / Wasmのcompile-tla / check-tla requestで明示指定し、名称はmaintainerへの相談事項とする。省略時は既存のv1出力・評価経路を使い、v1拒否をv2へ自動再試行しない。IRのversionと既存のrequest envelope / Wasm ABIのversionを混同しない。

既存golden、fixture、archiveとcompiler-evalの237入力（既知の235 matched・2 rejected）は変更しない。変更前にfrontendごとのv1 serialize結果を保存し、省略時・明示v1ともbyte-identicalを要求する。既存の正規化後IR比較・完全応答比較も維持し、新しい入力は別cohortに置く。run ID、revision、digestが変わる新manifest全体のbyte一致は要求しない。

通常引数は`{expression, caller lexical frame}`を持つcall-by-nameで、parameterを読むたびに評価し、値cacheは持たない。bound slotはframe内に限定し、parameter / captureと分離する。LETは定義位置の外側binder値・parameter thunkを明示captureにし、callerの引数をcalleeのbinderの下へ絶対slotのまま貼らない。current / next / primeはlexical frameから分離し、引数を要求した時点のphaseで評価する。EXCEPTのold scope、ENABLEDのfresh next候補・短絡順を保つ。level / constant / context-free / membershipの依存性は非再帰graph上でactual / captureを合成し、callを葉として判定しない。

実行時のcall entryと引数評価は毎回それぞれ`Arena::consume(1)`へ課金し、枯渇は`bounded/reason:work`とする。lower / 検証の制限と探索のmaxWorkを分ける。frontendの式nesting・bound variable上限40は上げず、定義内深さとcall経路長を混同しない。既存checkerの式48、値64、decoder128等も共有を理由に緩めない。

## 証拠から選ぶ範囲

提供済みの構造JSONの静的読取りによる内訳。ノード数は全面inlineを仮定した代数計数で、物理IRの生成結果・速度ではない。展開深さは`expandedPath`、定義内深さは全modeのcallを切った`callCutPath`、到達数は`reachedDefinitions`。最後の列は最深pathと各modeの寄与からの推論である。

| モデル | inline論理ノード | 展開深さ / 全call切断深さ | 到達定義 | Init/Next枝数 | Init/Next step総数 | 巨大化・40超の経路                                     |
| ------ | ---------------: | ------------------------: | -------: | ------------: | -----------------: | ------------------------------------------------------ |
| A      |            1,214 |                    17 / 8 |       25 |        1 / 10 |           11 / 140 | action外殻から代入右辺の式へ。指数規模・40超は観測なし |
| B      |      約1.7×10^18 |                   73 / 43 |      577 |         1 / 1 |              3 / 5 | guardの式定義・LET・actual複製。残る43も式の連言       |
| C      |      約6.1×10^82 |                  235 / 58 |    1,076 |        1 / 10 |           20 / 284 | action外殻から右辺の式定義・LETへ。残る58も式のIF列    |
| D      |      約3.9×10^20 |                   77 / 20 |    1,795 |         1 / 1 |              2 / 4 | 代入右辺の式定義・LET・actual複製                      |
| E      |      約3.6×10^15 |                   82 / 16 |      413 |         1 / 1 |              4 / 4 | 代入右辺の式定義・LET・actual複製                      |

今回は**expression定義の共有を最小範囲として選び、primed変数へ代入するactionはinlineを維持する**。B〜Eの巨大化と40超は式経路にあり、action固有の指数的な枝・step増大は資料に示されない。`mode=action`にはstate guardも含まれ、単にNextから到達することをaction共有の必要性とは数えない。定義表のbody kindは今回はexpressionを受理し、代入action templateは含めない。Init / Next / ENABLEDの既存branch展開内にもv2の式callを通し、primedな式・action predicateも検査する。

全call切断深さ8/43/58/20/16はexpression-only実装後の深さではない。action外殻込みの残存深さ・受理は未確認で、B/Cは全callを切っても40超のため全5モデルの受理を成功条件にしない。非公開ソースは追加せず、同じ5入力のv2出力の定義数、call edge数、serialized bytes、定義内深さをリポジトリ外で再計測し、匿名の数値だけを報告する。モデル名・ファイル名・識別子・所在を持ち込まない。式共有後もaction由来の指数増大が残れば仮説未達と記録し、action templateとENABLED列挙の共有を含む変更計画を結果・条件変更とともに先に登録する。action共有が一般に不要とは主張しない。

## 結果前に固定する回帰とRed

新規source/configとTLC goldenは後続で`evals/fixtures/tlc-operator-sharing/`へ保存し、既存fixtureディレクトリへ追記しない。共通moduleは`VARIABLE x`、`Init == x = 0`、`Next == UNCHANGED x`。設定は`INIT Init / NEXT Next / INVARIANT Safe / CHECK_DEADLOCK FALSE`を各行に置き、必要な`EXTENDS Naturals, Sequences, Apalache`を直接指定する。乱数・seed選別は使わない。

1. `D0 == TRUE`からk=1..60を具体的な名前で生成し、`Dk == IF TRUE THEN TRUE ELSE (D(k-1) /\ D(k-1))`、`Safe == D60`とする。k=10/20/30/60で定義数・call edge・serialized bytesの線形増加を検査する。現行両frontendに有限サイズのcompileとcompleteを要求し、nesting40の拒否を予想Redとして保存する。巨大なinline木を生成し切らない。v2はTLCと同じ正常結果を要求する。別sourceは`Dk == D(k-1) /\ D(k-1)`とし、D60の両枝を実評価するv2がmaxWork 256で`bounded/work`になることを要求する。

2. `Wrap(p) == \A c \in {9}: p`と`Safe == \A c \in {0,1}: LET G(q) == Wrap(q /\ (c \in {0,1})) IN G(\A z \in {0}: z = 0)`を固定する。現行でcompleteを要求し、checkerの`Invalid bound variable slot`を予想RedとしてTS/Rust両経路で保存する。同名cの誤captureを入れる故障版も同じ期待で落とす。別caseの`Safe == Wrap(E)`のEは、量化`\A z \in {0}: z = 0`、関数`[z \in {0} |-> z][0] = 0`、filter`{z \in {0}: z = 0} = {0}`、map-set`{z + 1: z \in {0}} = {1}`、fold`ApaFoldSeqLeft(Add, 0, <<0,1>>) = 1`と`ApaFoldSet(Add, 0, {0,1}) = 1`（`Add(a,b) == a+b`）、tuple pattern`\A <<a,b>> \in {<<0,1>>}: a+b = 1`に固定する。各caseはcaller binderをcallee binder下へ貼る現行の拒否を予想Redとし、未再現ならGreen対照と区別して記録する。

3. `Ignore(p) == TRUE; Safe == Ignore(Head(<<>>))`と、`Use(p) == p = 0; Safe == Use(Head(<<>>))`を固定する。未使用は正常、使用時は評価エラーの対照。現行は未使用引数を実行しないためGreenを予想し、全actualを先に評価する意図的な故障版でのみRedを示す。既存バグ発見と数えず、未使用の未対応構文まで受理範囲を広げない。

4. `Id(p) == p; Act(p) == x' = p; Prime(p) == p'`を定義し、`Next == Act(Id(1-x))`、`Safe == x \in {0,1}`、追加設定`PROPERTY P`で`P == [][Prime(Id(x)) = Id(1-x)]_x`を検査する。別caseはPを外し、Safeへ`ENABLED (Act(Id(1-x)) /\ x' # x)`と`~ENABLED (Act(Id(x)) /\ x' # x)`を連言する。currentとnextが異なる両状態、fresh候補、called prime / constant判定を検査する。現行で通るものはGreen対照、state固定・誤phase metadataの故障版でRedとする。自己cycle / 相互cycle / 不正definition ID / arity / capture / bound slot / phase違反のv2 IRを、最小正常IRの各1箇所だけを壊して固定し、compile可能な不正sourceも含め明示拒否させる。新IRの受理テストは現行のversion拒否でRed、拒否テストは正常IRが通る対照と対にし、version拒否だけで検証実装済みと数えない。

十分予算は新規小モデルでmaxDepth 64、maxStates 10000、maxWork 1000000。両枝D60は小予算の対照だけを実行する。課金境界は探索費用から独立に、式`Id(Id(TRUE))`を直接評価する最小IRでcall2回・parameter読取り2回のW=4を固定し、maxWork 3/4/5で枯渇/成功/成功を検査する。public探索でも同じ課金を使い、256予算でworkを超過せずboundedを返すことを確認する。実行監視はcompile / checker / TLCの各processを60秒で打ち切り、timeoutを予算結果や成功へ読み替えない。深いchainでもstack overflow / trap / panicを許さない。

## オラクル・一致条件・保存

意味論はTLC v1.7.4、tagのrevision `5a47802b5c391f59ecdd44117981f4ff8c0656ba`に固定する。[actualとcontextのLazyValue束縛](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/SymbolNodeValueLookupProvider.java#L81-L112)、[cache可否](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/value/impl/LazyValue.java#L37-L60)、[ENABLEDでのcache回避](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1473-L1489)を基準にする。TLCが常にcacheしないという意味ではない。期待値・計数・反例・評価エラーは同じsource/configのTLCログから確定し、idealis出力から作らない。jar SHA-256、実行環境、コマンド、source/config/log digest、stdout/stderr、終了code、エラー位置を保存する。TLCは1 worker、正常/違反/評価エラーを分け、反例は状態列を固定してTLCで再生する。

TS-v2 / Rust-v2は同じ定義ID・capture順・branch名/順・stepの正規化後構造を要求し、現行の空optional array / null ENABLED subscriptの省略以外で差を隠さない。native / Wasmは完全応答一致を要求する。v1で受理する入力は十分予算でv1/v2の状態集合・状態数・generated・値・反例・エラーを一致させる。v2専用の深いchain・scope修正はv1拒否を別に残す。新call課金によるwork量・小予算の部分結果の差は明記し、既存goldenを更新して消さない。2 rejectedを含む237件は既定v1で凍結確認し、v2を別runで照合する。

Red → Green → refactoringの順に、正常版・故障版・旧版・TLC対照と全失敗を新規runへ保存する。条件の変更を記録し、旧runを上書き・編集しない。`complete / bounded / empty`、未対応、TLCエラー、実行基盤失敗、timeoutを分ける。共有による構造増大の除去、scope修正、その入力での意味一致を判断し、実行時の指数仕事の除去、全TLA+互換、実プロジェクトの未知バグ発見、全5モデルの受理や速度改善を主張しない。

## 後続の検査とsupport記録

`just check`、`just browser`、`just compiler-test`、`just safety-test`、`just portable-build` / `just portable-test`、`just quint-test`、`just compiler-eval <新規run>`、`just evaluate` / `just replay-evaluation`を実行する。保存入力の再投入、`compiler-tlc`、`safety-eval` / `safety-replay`、既存TLC differential goldenと反例再生も確認する。共有テストまたはmutation対象変更時は`just mutation`。rtlc / rtla-compilerはそれぞれcargo fmt --check / clippy --all-targets --locked -- -D warnings / test --all-targets --lockedを確認し、portable core変更分のfmt / clippyと実Chromiumも検査する。

変更するrtlc / rtla-compiler（request変更時はidealis-coreも）のSUPPORT / READMEへv1既定、明示v2、共有・遅延引数・制限を反映し、`just support-plan`で対象を確認する。`scripts/check-support.ts:288-300`はcrate内の全Rustソース等をmanifestへ含め、同`:346-350,467-476`は現bytesのsnapshotへ保存reportを照合するため、ソース変更後は古い記録でoffline CIを通せない。キーなしのskipはlive syncだけ（同`:394-404`）で、CIの通常検査はskipしない（`.github/workflows/ci.yml:23`）。

当方にJev API keyはない。後続PRのCIをgreenにするには、キーを持つmaintainer側で`just support-sync`（部分stageは`support-sync-staged`）を行い、判定を確認・必要なら修正して再同期し、SUPPORTと`.jev-lint/checklists`のmanifest / recordをstageして`just support-check-staged` / `just support-check`を通す。skipを同期済みと報告せず、記録・digestを手編集しない。この計画ファイルだけの追加は既存manifestの対象外なので、保存記録を変更しない。
