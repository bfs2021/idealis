# 0039: feedbackまで届く入力でprotocolをfuzzする

生JSONLと、pending ID・caseを使う構造付きcommandを、cargo-fuzz/libFuzzer/AddressSanitizerで検査した。protocolの状態と保存caseを検査する回帰targetとして採用する。公開coreや配布物は変更していない。[事前計画](../plans/0039-structured-fuzz.md)と[実行・再生手順](../structured-fuzz/README.md)を参照。

## 実行した範囲

| run                                                | 各targetのrun数 | raw corpus | structured corpus | crash artifact |
| -------------------------------------------------- | --------------: | ---------: | ----------------: | -------------: |
| [開発](../runs/0039-fuzz-development/result.json)  |          10,000 |        947 |             1,255 |              0 |
| [確認](../runs/0039-fuzz-confirmation/result.json) |          10,000 |        962 |             1,220 |              0 |

計40,000 runsで、新しいpanic・ASan違反・観測したprotocol状態破損は見つからなかった。保存corpusは計4,384入力で、全件をASanの固定入力として再実行した。通常nativeとWasmでは、追加probeも含めて計5,152記録・112,720要求/応答を再生した。成功応答は一致し、不正入力も両方でerrorとなった。

sourceVersionは`a918bae9465e5991f4a8db5e3aa8dd57be41637b780fedbca24621d5a94d8ff4`で共通。開発seed 0x39a1、確認seed 0x39c3で、同じ初期corpusから別々に開始した。source・lock・引数・binary hashと全保存入力を残している。新しいcampaignで同じcorpusが選ばれることは保証しない。

## 同じbytesでの到達性

campaignとは別に、同じ乱数bytes列を両targetへ渡した。表は終端応答を1回以上観測した入力数。入力全体や全sessionの処理完了数ではない。

| probe | 入力数 | rawの終端到達 | structuredの終端到達 | structuredの縮小phase到達 | structuredのbounded |
| ----- | -----: | ------------: | -------------------: | ------------------------: | ------------------: |
| 開発  |    128 |             0 |                   90 |                        42 |                   0 |
| 確認  |    256 |             0 |                  185 |                        97 |                   4 |

構造付き生成は早期のJSON拒否を避け、feedback・縮小・確認の経路を検査できた。一方、既存の有効traceでseedしたraw corpusも、開発433/947入力、確認419/962入力が終端へ到達した。構造付きcorpusはそれぞれ1063/1255、1011/1220入力だった。これらは異なるcoverageで選ばれた集合で、検出率やthroughputの公平な比較ではない。target間のLLVM coverage値も直接比較していない。

corpusのboundedはrawが開発19・確認42入力、structuredが280・242入力。入力長、command数、自動feedback、Engine呼出数を制限したprefixの検査である。全部のcommandを実行した成功へ読み替えない。新しい実SUTの不具合発見もない。

## 状態破損の対照と再生

不正requestを受けたsubjectと、その要求を除く履歴から復元した別Engineを比較する。同じ実装のmetamorphicな検査で、共通する意味論の誤りをすべて検出できるものではない。pending sessionを意図的にcancelする既知故障の対照は、次の有効feedbackで不一致を検出した。これは照合器が状態破損を検出することを確かめる人工故障で、未知バグには数えない。

保存case・wrong ID・extra field・JSON拒否・cancel、生成・縮小・確認・再生、比較とschedule構築を含む。Rustの3 testsと共通2 testsをnode:test/Vitestで実行した。実Chromiumは4つの固定transcriptを再生し、ホストimportとcross-origin isolationは不要だった。ブラウザでASanや全40,000 runsを動かした結果ではない。

`just check`、`just browser`、runtime-build/test、evaluate/replayを確認した。[検証run](../runs/0039-fuzz-verification/result.json)に成功と開発中の失敗ログを保存する。初回の到達性testではgeneratorのminLengthが全長となり削除候補がなく、縮小phaseへ届かなかった。通常planの最小長を1以下にして、同じassertionを通した。coreの不具合として扱っていない。

## 費用と採用判断

全validationは開発約44秒、確認約47秒。2 targetのfuzzing、corpusのASan再生、通常native/Wasm照合を含む。libFuzzer報告のpeak RSSは479-536 MiBで、ASanの費用を含む。通常coreやWasmのメモリ使用量とは別の指標。

Wasmは0038と同じ374,454 bytes、SHA-256も同じ。nightly-2026-08-27、cargo-fuzz 0.13.2、libfuzzer-sys 0.4.13、Arbitrary 1.4.2をprivateなevalへ固定した。公開coreの依存・APIとpackageは変更していない。

追加到達と保存入力の再現を確認できたため、boundedな開発・CIの回帰targetとして採用する。bug不在や全入力の正しさの保証には使わない。構造付きmutationとgenerationの比較、形式言語parserへのgrammar fuzzing、より長い状態履歴は別の仮説と予算で評価する。
