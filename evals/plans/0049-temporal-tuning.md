# 0049: 時間論理のprofileとmicro tuning

## 仮説と固定条件

0048の量化fairnessを対象に、CPU profileで支配的な処理を特定する。公開IR/API、演算とerrorの評価順、探索順、state/generated/work、予算境界と反例を保つ。新機能・探索の枝刈り・公平性の意味変更は範囲外。

比較元は開始時のcommit `0e9212ac`。変更前のnative/Wasmを別pathへ保存し、hashと入力を固定する。速度だけを変える候補の挙動は変更前でも通す。新たな資源制約や意味論を導入する場合はRedから回帰を追加する。

## 探索

native releaseのprepared IRを各10,000回checkする。固定入力はoperators-support.jsonのfair-forall、fair-intermittent-WF、fair-intermittent-SF。parse/compileは反復前に行う。samplyを1,000 Hzで使い、source/binary hash、入力、全結果の一致、raw profileとself/inclusive sampleを保存する。profile付きの時間を改善率に使わない。

profileを読んだ後、対象処理と候補を実装前にこのplanへ追記する。profile/build/testとCriterionを並走させない。プロファイラの失敗はそのまま保存し、必要な条件変更を記録する。

## 計測と採用

compilerの既存27本をfresh before/afterで測る。20 sample、warmup 0.5 s、measurement 1 s。ASTからのcompileとprepared IRのcheckを分け、parser/IPC/起動を含めない。rtlcの既存14本もbefore/afterの対照にする。各runは新しいdirectoryへ保存する。3%を超える後退が見えた対照は再測定し、残る後退を記録して採否を決める。一般のモデルやTLCとの速度比へ外挿しない。

改善後も同じ3入力のprofileを取り、実行回数とsample数を併記する。割合の変化だけで採用しない。境界64/65等のformula数、分岐のclone、acceptanceのpending、空domain、無効なIRを対照にする。

## 回帰と記録

52ケースのnative/Wasm/TLC判定・計数と4反例の再生を検査する。変更前binaryとも完全な応答を比較し、workを含む結果の差を許容しない。小さいwork/state/depth/temporal予算と評価エラーも比較する。Chromium、crate fmt/clippy/test、just check/browser、portable-build/test、safety-test、runtime-build/test、evaluate/replay、package consumerとSUPPORT/Jevのstaged検査を通す。runを編集せず、採用と保留の判断をiterationへ記録する。

## profile後の候補（実装前）

[変更前profile](../runs/0049-profile-before/result.json)のSFは11,245 sample。HashMap<usize, ()>::insertはinclusive 1,646 sample、Expansion::cloneは1,319 sample。WFもそれぞれ1,182/1,133 sampleだった。訪問済み式のHashSetが小さい整数のhash・rehashとcloneを発生させている。

候補はExpansion.seenだけの変更。poolが256式以下なら[u64; 4]へ、257式以上なら従来のHashSetへ保存する。疎な大きいpoolの分岐ごとに全pool分のbitmapを確保しない。挿入・所属だけを置き換え、todo/next/pendingの順序とarena.tick、SCC・witnessの処理は保持する。16/17/64/65要素の時間論理forallでword境界とfallbackの両側、正常・違反・予算境界を比較する。高速化のためにテストの意味を変えない。
