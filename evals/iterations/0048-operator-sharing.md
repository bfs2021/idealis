# 0048: operator共有の回帰とRed

最初の条件訂正として、[計画](../plans/0048-operator-sharing.md)の既存compiler入力237件（235 matched・2 rejected）を、base revision `fbf0d664`の`evals/rtla-parser/corpus.ts`が返す239件へ訂正する。`git log`と`evals/tlc-differential/corpus.ts`の差分では、`04946491`（#53）がfold、`a733d37d`（#54）がfunction orderの境界入力を追加し、同#54の最終修正で`fold-equality-entry-error`と`function-equality-entry-error`の2件が追加されている。先行報告の237件はこの最終2件を含んでいなかった。v2の結果を見る前に件数239をassertし、現行の各frontendのserialize結果と拒否を新規のv1 baselineへ保存する。回帰の4 familyと、それ以外の計画条件は変更しない。計画原本・既存fixture・保存runも変更しない。

保存時の実測はTS/Rustとも237受理・2拒否だった。追加17入力も同時に固定した。[fixture](../fixtures/tlc-operator-sharing/README.md)の先行19ログは全digestを検証し、元のraw logへ指定の置換だけを再適用して同じbytesになることを確認した。jar/moduleも指定SHA-256と一致し、private stringの検査を通過した。

先行実行の条件変更として、family 4のprime / ENABLEDでは共通のNextをActの宣言前へ置くと、TLCが終了150でforward referenceを拒否した。その2入力と拒否ログを保持し、修正版はId / Act / Primeの宣言順だけをNextの前へ移した。式・遷移・性質・予算は変えていない。v2の結果を見る前の変更である。

v1の観測ではchain 10は正常、chain 20/30/60とwork-60はnesting 40で拒否された。lexical captureとbinder 7件はcheckerのslot検証で拒否された。foldの2件だけは`Invalid fold variable slot`、残る6件は`Invalid bound variable slot`であり、Green対照とは数えない。未使用引数、prime、ENABLEDはTLCと同じ正常結果、使用引数は空列Headの評価エラーだった。TLCのエラー位置・全文はgoldenへ保存し、既存frontendのHead診断には位置がないため意味上のエラー分類を照合する。

初回のsafety-testでは想定した110件のRedに加え、人工的なprime故障の検出対照が失敗した。故障生成器がpropertyのstuttering条件までcurrent stateへ固定し、`x = x`として常に真にしたためだった。stutteringの右枝を保持し、called primeの式だけを誤phaseにする故障へ修正した。入力・TLCオラクル・成功条件は変更せず、初回ログも保持する。この人工故障は既存バグ発見とは数えない。

step 3bのv2 frontend結果を見る前の条件訂正として、operator-sharing cohortのv1 24件を、現在のv1が返す型付き拒否の検査へ変更した。TS/Rustの現在挙動を各入力で再採取し、chain 20/30/60とwork-60の8件は`protocol` / `Unsupported TLA input: expression or operator nesting exceeds 40`、lexical captureとquantifier/function/filter/map-set/tuple-patternの12件は`protocol` / `Invalid bound variable slot`、fold sequence/setの4件は`protocol` / `Invalid fold variable slot`を完全一致で要求する。v1のbyte-identical条件と、v2専用の改善ではv1拒否を別に残す計画に合わせた訂正であり、TLC結果をv1の成功へ読み替えない。他のv1条件と全v2のTLC期待値、既存239入力、計画原本、保存TLCログとexpected.jsonは変更しない。

step 3bでは両frontendの明示v2生成を実装した。expression定義は字句宣言位置・phase・membership用途をkeyとして一度だけlowerし、最終IDを固定順へ並べ直す。binderはframe内のslot 0から始め、parameter / captureと分離した。captureは同名でも字句binding IDで区別し、見えなくなった外側bindingとsibling LET経由の依存も投影する。定義lowering・capture圧縮・metadata解析を値のmemoizationと分け、actionは代入を生成する外殻のinlineを保ち、式guardへshared callを通す。native/Wasmのsource入口に残っていたv2未実装guardを外し、今回のcohortとして先に追加したversion配線テストも正常受理へ移した。既存239入力のテスト条件は変更していない。

実装途中では、v2のAnd/Orへv1由来の補助TRUE/FALSEを含めると、両枝D60がwork予算より先に動的深さ128へ到達した。v2だけ補助ノードを省き、制限を変えずにmaxWork 256のbounded/workを得た。rangeの型所属は、共有後に巨大なmap-setを全列挙する後退を避けるため、v2のidentity range map-setを所属判定時だけdescriptorとして扱う。通常値map-setとv1は従来の列挙を維持する。v2 decoderはENABLEDのnull subscriptを保存し、v1の省略処理を維持してnative/Wasmの完全応答を揃えた。

人工的な誤capture対照では、captureを9へ固定すると定数の不変条件が偽になり、探索前にprotocolの`Constant invariant or property is false`で拒否された。当初の「invariant-violationを返す」という対照側の分類を、この厳密な型付き拒否へ訂正した。正常版の受理・TLC期待値は変えず、任意の拒否やversion拒否を検出へ読み替えていない。初回失敗ログは保持し、この故障を未知バグ発見へ数えない。

最終のoperator-sharing cohortは426成功、失敗・skip・TODOは0。17保存入力の両frontendによるTLC対照、構造一致、native/Wasm応答一致、IR拒否、人工故障、課金境界と線形成長を含む。k=10/20/30/60の定義数は12/22/32/62、program全体のcall edgeは22/42/62/122、serialized bytesは両frontendとも4712/8682/12652/24562だった。両枝D60はbounded/work、使用引数はHead(empty)の評価エラーとして区別する。追加の手書きsource対照はTLC由来の独立入力へ水増ししない。

既存239入力を新規run [0048-operator-sharing-step3b](../runs/0048-operator-sharing-step3b/result.json)で評価し、237 matched・2 rejected、失敗0だった。既定・明示v1の保存digestは239＋追加17入力の全6経路で不変。追加のv2差分検査でも、237受理入力のTS/Rust正規化構造とcheck結果が一致し、native/Wasmのcompile/check応答は239件すべて一致した。有限予算でのv1/v2意味結果はwork欄を除いて239件一致した。call課金によりwork量が変わることを出力同一とは扱わない。

必須のfmt/clippy/testはrtlc 51件、rtla-compiler 41件、idealis-core 18件が成功。`just check`は154件、`just safety-test`は639件、`just portable-test`は132件、`just quint-test`は13件、`just browser`は3件が成功した。portable-testの132件には実Chromium 2件を含む。`just portable-build`、`just compiler-test`、`just support-plan`と`just evaluate` / `just replay-evaluation`も成功した。support-planは7 crates / 28 comparisons。support-syncはJev API key未設定の警告skipで、同期・オフラインsupport検査済みとは数えない。保存support recordやdigestは手編集しない。計画原本・保存fixture/TLCログ/expected・v1 baselineは不変で、stage、Git履歴・参照、GitHubの変更は行っていない。

step 3cの修正前条件を追加する。5000定義の前向きchainをTS/native/Wasmでcompile-only比較し、同じ受理構造を要求する。guard・prime・membershipの長いchainと循環拒否も検査し、定義graphを関数再帰でたどらない。UNCHANGEDのtuple別名・parameter・nested別名の3入力を別cohortへ追加し、v1/v2の各経路を新規TLCログと照合する。既存expectedとbaselineは変更しない。実仕様Bの費用については、LETスコープによる定義AST複製を疑い、独立した合成入力（200定義、各64要素tuple、100 sibling LET）で両frontendの構造と各compile 5秒以内を要求する。CASEは1/48/127/128/129/169 armを合成し、v1/v2のlowerとdecode/checkの境界を区別する。上限は変更せず、既存decoder上限に由来する拒否は両v2 frontendへ同じ型で伝播させる。非公開の内容・識別子は回帰や文書へ使わない。

初回のscope費用対照はTS約34ms・修正前Rust約97msで、複製経路を通るが5秒の境界を再現しなかった。そのため費用回帰の条件を結果前に1000定義×64要素・1000 sibling LETへ拡大し、5秒の上限は維持する。小さい対照の結果は保持する。

step 3cの途中で行った実仕様Bの短い再測定はテスト実行と重なる可能性があったため、費用比較の正式結果には採用しない。最終の5入力測定は全検査の後で、他のモデルを読み込むcommandと重ねず一件ずつ行う。短いprocessでは1秒間隔のwatchdog標本がピークを取り逃がすため、コピーした測定scriptに`time -l`のprocess最大RSSも追加し、標本最大と区別する。

step 3cではRustの循環検査を明示stackへ、level解析を依存順のparameter/capture summaryへ、generative判定を明示task stackへ変更した。5000定義のinvariant / guard / prime / membershipはTS/native/Wasmでcompile-only構造が一致した。TRUEへ至る5000 chainと変数targetへ至る5000 chainも5001定義で一致した。checkerの5000 call対照はnative/Wasmとも動的128のprotocol拒否で、stack overflowではない。capture固定点/pruning、TSのlevel/generative/lower queue、rtlcの依存順validationとconstant/context-free/membership metadataは、call ID先のbodyへ関数再帰で降りない。inline/action/temporalはfrontend40、validationは式48、評価/引数強制/membershipは動的128で制限する。

費用増大の原因はLETのscope作成が既存Entryの宣言ASTまで深くcloneしていたことだった。scopeはRc<Entry>、宣言はRc<Definition>でimmutableなASTを共有する。合成1000定義・1000 LETは修正前Rustが60秒timeout、修正後はTS約261ms・Rust約838ms、2001定義・3716373 bytesで構造一致した。cohortでは各compile 5秒以内を要求する。実仕様Bの修正前Rust v2は約82.6秒・watchdog最大3013856 KBでcompiled expressionの拒否だった。修正後は下表の費用で両方が`Unsupported TLA input: compiled expression nesting exceeds 128`を返し、受理やchecker成功へは数えない。値memoization、再帰、action template共有は追加しない。

UNCHANGEDの別名・parameter・captureを明示frame配列で変数/tupleへ解決し、ENABLED内だけで参照される別名とSPECIFICATION subscriptにも適用した。生成的/式のUNCHANGED、直接/別名/parameterのprime、BoxAction/AngleActionとfairnessのsubscript、代入/選択target、ENABLEDのbranch生成を点検した。代入左辺のsyntactic Identifierという既存制限は維持する。新規3入力は固定TLCから各success・1状態/生成2を取得し、追加goldenへ分離した。既存expected・ログ・v1 baselineは変更しない。

169 arm CASEはv1のcheckerでもJSON decoderの`recursion limit exceeded`を再現した。同じCASE本体をv2 envelopeへ入れたchecker対照でもnative/Wasmが同じ拒否だった。CASE loweringとchecker上限は維持する。v2では既存compiled expression decoder128超過を両frontendで同じ型の早期拒否へ伝播させ、Bの従来TS受理/Rust拒否を両拒否へ揃えた。これは既存checker制限の解消ではない。

最終cohortは469件、v1 byte guardは元の239＋17入力を含む257件が通過した。省略/明示1・TS/Rust/nativeの1536 digest比較は不変。新規run [0048-operator-sharing-step3c](../runs/0048-operator-sharing-step3c/result.json)は239入力・237 matched・2 rejected。追加v2照合は237構造一致・既知2拒否、239 native/Wasm compile/check一致、work以外の意味結果239一致だった。補助scriptの初回は既知2拒否の文言差を完全一致として失敗したため、元のcompiler評価に保存されている2件それぞれの診断を固定して確認した。新たな拒否を合格へ置き換えず、元の2件の文言差を保持する。

必須fmt/clippy/testはrtlc 51件、rtla-compiler 45件、idealis-core 18件が成功。compiler-test 45件、safety-test 682件、check 154件、browser 3件、portable-test 136件（実Chromium 2件を含む）、quint-test 13件、support-plan 7 crates/28 comparisons、portable-build、evaluate（20固定seed・PBT 12 records）とreplay-evaluation（人工故障5件・schedule6件）も成功した。途中のcheckはiterationの空行formatだけで失敗し、整形後の再実行で解消した。保存runは編集せず、台帳はevaluatorが追記した。Jev support-syncは実行不可のため今回は未実行で、同期済みやsupport-check済みとは数えない。保存support record/digestとindexは変更しない。

5仕様の全20 commandは一件ずつ、watchdogの4 GB相当RSS上限・300秒・Node heap 2048 MiBで実行した。短命processのwatchdog標本最大は真のピークではないため、以下のRSSは`time -l`のprocess最大MiBで、監視打切りだけwatchdog KBを示す。v1も受理時は同じ有限checker予算で検査した。

| 匿名入力 | v1 TS / Rust                    | v2 TS / Rust                    | v2 bytes / definitions | v2 compile ms TS / Rust | v2 peak MiB TS / Rust |
| -------- | ------------------------------- | ------------------------------- | ---------------------- | ----------------------- | --------------------- |
| A        | 受理・deadlock / 受理・deadlock | 受理・deadlock / 受理・deadlock | 30972 / 20             | 7.4 / 18.0              | 97.2 / 104.2          |
| B        | 拒否40 / RSS打切り 4010016 KB   | 拒否128 / 拒否128               | —                      | 252.4 / 333.5           | 275.1 / 213.4         |
| C        | 拒否40 / 拒否40                 | 未対応式拒否 / 未対応式拒否     | —                      | 39.6 / 39.6             | 114.4 / 103.4         |
| D        | 拒否40 / 拒否40                 | 拒否40 / 拒否40                 | —                      | 34.1 / 58.4             | 111.1 / 109.9         |
| E        | 拒否40 / 拒否40                 | capture拒否 / 拒否40            | —                      | 48.0 / 75.8             | 115.6 / 115.1         |

Aのv1は両frontendとも61610 bytes・定義0、v2は30972 bytes・20定義で、いずれもdeadlock・325状態・深さ9だった。workはv1 37395、v2 87360で、call課金の差を保持する。拒否時のbytes/definitions/checkは未返却。CとEの両frontendはprotocol拒否だが診断文言・最初に検出する非対応条件は同一ではなく、受理構造の一致や全5入力の成功は主張しない。B Rust v1の192秒・RSS打切り1件は既存経路の未解決な測定失敗として保持し、typed refusalや成功へ変換しない。今回の費用修正や人工入力での検出を実プロジェクトの未知バグ発見には数えない。

最終対象位置監査の追加Redとして、`Can(p) == ENABLED (UNCHANGED p)`とLET版がv1で受理されv2で両拒否されることを確認した。共有frameのformalから生成的代入slotを決める外殻が原因で、parameter/captureを持ち、ENABLEDとUNCHANGEDに依存する外殻は既存actionと同様にinlineする。式のactualと通常のexpression定義は共有を保つ。外殻の判定はsource定義graph上の反復metadataで行い、call chainの関数再帰は増やさない。直接formal・推移的formal・capture・LET formalの4対照を追加し、以降の全検査と測定を再実行する。先行run/測定は保持し、追加runを別名で保存する。

外殻inlineの追加監査で、foldのoperatorへも適用すると専用frameのaccumulatorとbinderが混線するRed（v1 complete / v2 Invalid safety binding count）を確認した。foldの明示actual呼出しは共有frameを維持し、通常のsource呼出しだけで外殻をinlineする。ENABLEDとbinderを持つfoldのv1/v2対照を追加した。先行runは保持し、この境界修正後をstep3c-final runと測定先へ別保存する。

foldの新対照は、v2専用だったcompileParity helperをv1にも使ったため、既存v1 decoderのENABLED null subscript省略で応答完全一致が失敗した。v1のこの追加対照だけ既存のoptional/null正規化ルールに合わせ、v2の完全応答一致と元のv1 byte基準を維持した。product codeはfold境界修正から変わらず、20測定は同じcompilerの結果を使う。補助test修正後のsource snapshotはstep3c-verified runへ別保存する。

## generative positions under v2

step 3d-aは設計とRed入力の追加だけを行う。第二回レビューで、step 3cの生成的外殻inlineが通常actualの字句frameを失うことが判明した。`F(v,a) == IF ENABLED (UNCHANGED v) THEN [<<4>> EXCEPT ![1] = a] ELSE <<0>>`を外側EXCEPTから`F(x,@)`で呼ぶと、actualのold=9がcalleeのold=4へ変わる。同じ置換はbinderのslot混線とTSのmembership用途の消失も起こす。外殻にENABLEDがあるか、targetがformalへ依存するかでinline範囲を狭めても、通常actualを移動する問題は残る。ここで外殻全体へのIR代入方式を撤回し、以下を後続実装の設計とする。既存実装や[IR文書](../../docs/safety-ir-v2.md)にある外殻inlineの記述は現状の説明であり、この設計の実装済みという意味ではない。

静的解決は、生成的UNCHANGEDのtargetと、ENABLEDのbranch生成・action subscriptに必要な変数位置だけへ限定する。resolverの入力は式単独ではなく`(source expression, declaring/caller lexical environment)`とし、state variable、変数tuple、別名、formal、captureを宣言identityで追う。formalのactualやcaptureをたどるたびに元のcaller/宣言frameへ戻る。結果は状態slotまたはそのtupleという構文上のdescriptorだけで、実行時の値・binder値・old値を静的評価しない。通常の計算式、binder、`@`、定数値等を変数targetとして要求された場合は`protocol`の型付き非対応を返す。これは現在扱う有限fragmentの制限であり、TLCが任意のUNCHANGED式を拒否するという主張ではない。

通常actualとcaptureは全て既存v2の`call` / `parameter` / `capture`とimmutableなcaller frameに残す。同じformalをtargetと通常式の両方で使う場合も、target位置だけdescriptorへ解決し、他の参照はparameterのままにする。calleeにbinderやEXCEPTが増えてもactualのlocals/oldをcalleeのものへ付け替えない。current/next/primeとENABLEDのfresh candidateは動的contextを維持する。事前の値評価、値cache、binder slotの付け替えだけによる修正は採用しない。

定義をcallerなしでlowerするだけでは`Can(p) == ENABLED (UNCHANGED p)`のslotを決められないため、依存順に「静的targetが要求するformal/capture」のsummaryを求める。call siteで必要なactualだけをcaller frame付きで解決し、宣言位置・phase・purposeに**解決済みtarget signature**を加えてexpression定義をspecializeする。同じsignatureは共有する。signatureはtarget出現順とtupleの順序・形を保つstate slot列で、値やframe IDをkeyにしない。通常parameter/capture数・参照は維持し、未使用actualも既存の構文・IR検証を通す。IDの順は既存の宣言/phase/purpose順にsignatureの構造順を加え、TS/Rustで一致させる。signature増加は定義数・bytesとして計測し、異なるtargetを同じbodyへ誤共有しない。既存のnesting/decoder/work上限は変更せず、非再帰graphの明示stackと循環拒否を維持する。解決できない場合は外殻inlineへ戻さない。

| 位置                                                                     | v1 frontendの現状を維持                                               | v2 frontendの設計                                                                                                                                                        | 固定TLC 1.7.4の根拠・境界                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ------------------------------------------------------------------------ | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 生成的UNCHANGED target                                                   | 既存inlineと変数/tuple別名解決。保存bytes・拒否を維持                 | この位置だけcaller frameをたどって変数/tupleへ解決しassign slotを生成。非変数は型付き拒否                                                                                | [processUnchangedImpl](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1279-L1369)は変数/tuple/0-arity別名とLazyValueのcontextを特別扱いし、その他は評価比較へ進む。式としてのUNCHANGEDは[評価比較](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2309-L2314)を保つ                                                                                                                                                                                                                                                                                       |
| ENABLED body                                                             | 既存action branch展開とinline                                         | fresh next候補を生成するaction構文は展開するが、本体全体・通常actualは代入しない。内部の生成targetだけ静的解決し、guard/RHSはframe付き式参照にする                       | [ENABLED入口](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2149-L2155)、[enabledのoperator/LazyValue](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2588-L2618)、[enabledUnchanged](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2975-L3030)。ENABLED bodyそのものを変数tupleとみなさない                                                                                                                                                              |
| `[A]_v` / `<<A>>_v` subscript（SPECIFICATION・property・fairnessを含む） | SPECIFICATIONは変数列を要求し別名解決。通常の式subscriptは値比較      | branch生成/SPECIFICATIONが変数位置を要求するvだけcaller frameから変数/tupleへ解決し、解決不能なら型付き拒否。通常の式subscriptはframe付き値比較を維持。A中の通常式は共有 | [next-state生成](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1101-L1116)、[enabled angle](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2653-L2657)、[enabled square](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2785-L2792)。TLCの[通常評価](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2315-L2343)は一般のeの値も比較でき、v限定はidealisの範囲 |
| ordinary expression / actual / capture                                   | 従来の式inline。既存の誤束縛もbaselineとして別記                      | caller frameを持つcall-by-name参照。生成位置との同居を理由にinlineしない                                                                                                 | [getVal/getOpContext](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/SymbolNodeValueLookupProvider.java#L86-L120)と[LazyValue評価](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1530-L1542)に基づくframe保持。TLCのcache全般を禁止する意味ではない                                                                                                                                                                                                                                                                                                                 |
| membership set                                                           | 既存membership lowering。`Types(Nat)`/`Types(N)`のv1はNatを型付き拒否 | actualがこの位置へ達すると両frontendともmembership purposeでlowerし、alias/formal/captureの全経路へ伝播する。valueへ落としてから貼らない                                 | [membership評価](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2241-L2246)を満たすdescriptorはidealis側のlowering方針。TLCに同名のspecialization方式があるとは主張しない                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| binder domain                                                            | 既存の有限domain loweringとslot制限                                   | caller frameでdomainをlower。calleeのbinderはframe-local slotで導入し、actualのbinderをその下へコピーしない。列挙domainをmembership型集合と混同しない                    | [contexts](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L3471-L3523)と[enabled quantifier](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2659-L2688)のcontext分離                                                                                                                                                                                                                                                                                                                                                                                       |
| EXCEPT value with `@`                                                    | 既存inline。今回のv1誤束縛はTLC適合と数えない                         | 更新式自身のoldと、actual/captureのcaller oldを分ける。後者はthunkのframeに保存                                                                                          | [EXCEPT更新context](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1871-L1897)と[@ lookup](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1440-L1443)                                                                                                                                                                                                                                                                                                                                                                                                     |
| LET body                                                                 | 既存inlineとlexical scope                                             | 宣言位置のparameter thunk/binder/oldを明示captureし、通常参照は共有。targetに要求されたcaptureだけ元frameで静的解決                                                      | [LET評価](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1421-L1425)と[enabled LET](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2485-L2498)                                                                                                                                                                                                                                                                                                                                                                                                            |
| CHOOSE / set-comprehension body                                          | 既存の有限binderとinline                                              | predicate/elementは通常のframe付き式。binder導入前のactualと導入後のbodyのframeを分ける。生成位置が内部にあっても全bodyへactualを貼らない                                | [bounded CHOOSE](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1686-L1729)、[set comprehension](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2008-L2066)                                                                                                                                                                                                                                                                                                                                                                                               |

action/temporal外殻のAST展開と、expression actualのIR置換は別に扱う。前者を維持することは、後者によるframe移動の例外を許す意味ではない。action guard・代入RHS・temporal predicateへ渡る式も、宣言環境を保持して通常のshared expressionとしてlowerする。現在のexpression-only wireでcaller frameを保持できないaction形は、型付き非対応として明示し、raw IR代入やaction templateの暗黙追加で回避しない。この段階で任意のaction引数・全TLA+の受理を約束しない。foldのaccumulator/element専用frameも同じ不変条件を守る。

結果前に6入力を固定する。EXCEPT再現は末尾`= 4`と正常対照`= 9`、binder再現は`Can((\A y \in {0}: y = 0))`と同じCanへTRUEを渡す対照、membershipは`Types(Nat)`と`N == Nat; Types(N)`。全て共通の`Init == x = 0` / `Next == UNCHANGED x`とし、binder/EXCEPTのSafeには`x = 0`を含める。新しいtiny fixtureだけを固定TLCへ一件ずつ投入し、新規expectedへログ由来の判定・計数・初期反例を保存する。v1は現状の結果を別のassertionで固定し、v2はTLC意味結果・TS/Rust構造・native/Wasm完全応答を要求する。正常対照の`= 9`も現v2では偽違反になる可能性があり、成功へ反転せずRedとして残す。既存のUNCHANGED別名/parameter/nested3入力・goldenとv1保存digestは不変とする。

最初の6ログとcompiler観測後の条件追加を記録する。binderへ加えた`x = 0`の連言により、v2の最初の診断が`Invalid safety variable read`となった。先行のguard付き再現・対照とログは保持する。レビューの`Safe == Can(...)`そのものも固定するため、連言だけを除いた`sharing-generative-binder-exact`とTRUE対照を追加する。後者のTLC結果を見る前にこの2入力を固定し、合計8入力を検査する。レビューと同じ形のv2診断は`Invalid safety binding count`であり、いずれもv1は`Invalid bound variable slot`で拒否する。guard付き入力を隠す置換やgoldenの更新は行わない。

### actual / captureの同型箇所のsweep

現在の両frontendで、`call` / `inline_call` / `sharedCall` / `shared_call`、locals/projectionへのinsert、actual/captureのclone・map、fold・LET・binder・old・pruning・renumberを検索した。以下はstep 3d-a開始時の実装行番号で、`fixed-by-design`は後続修正の対象、`not the same type`は式をcaller frameから移す処理ではない、またはv1凍結という分類である。今回修正済みという意味ではない。

| 箇所                                       | TS (`src/tla-safety.ts`)                                                                                             | Rust (`crates/rtla-compiler/src/lib.rs`)                                                                                                                                                     | disposition                                                                                   |
| ------------------------------------------ | -------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| generative outer-shell分岐                 | src/tla-safety.ts:373–380                                                                                            | crates/rtla-compiler/src/lib.rs:604–644                                                                                                                                                      | fixed-by-design: 外殻全体への代入を廃止しtargetだけ解決                                       |
| inline helperのcapture再投影               | src/tla-safety.ts:597–605                                                                                            | crates/rtla-compiler/src/lib.rs:923–940                                                                                                                                                      | fixed-by-design: 生IRをcallee localsへ貼らず宣言frameを保持                                   |
| inline helperのactual代入                  | src/tla-safety.ts:613–624                                                                                            | crates/rtla-compiler/src/lib.rs:948–954                                                                                                                                                      | fixed-by-design: callerでlowerしたIRだけの移動も禁止                                          |
| Rust外殻専用capture代入                    | —                                                                                                                    | crates/rtla-compiler/src/lib.rs:613–628                                                                                                                                                      | fixed-by-design: helper外の重複実装も対象                                                     |
| Rust外殻専用actual代入                     | —                                                                                                                    | crates/rtla-compiler/src/lib.rs:631–637                                                                                                                                                      | fixed-by-design: helper外の重複実装も対象                                                     |
| generative action inline                   | src/tla-safety.ts:1602–1607                                                                                          | crates/rtla-compiler/src/lib.rs:1863–1875                                                                                                                                                    | fixed-by-design: AST外殻と式のframe保持を分離                                                 |
| temporal / boxed-action展開                | src/tla-safety.ts:1818–1823, src/tla-safety.ts:1833–1838                                                             | crates/rtla-compiler/src/lib.rs:2040–2045, crates/rtla-compiler/src/lib.rs:2072–2077                                                                                                         | fixed-by-design: actual付きhelper経路も式参照を維持                                           |
| root / SPECIFICATION alias展開             | src/tla-safety.ts:1795, src/tla-safety.ts:1953–1954                                                                  | crates/rtla-compiler/src/lib.rs:2160–2161, crates/rtla-compiler/src/lib.rs:2225                                                                                                              | not the same type: 現用途はactualなしのroot展開。capture付きへ拡張するならframe契約が必要     |
| shared definitionのformal/capture生成      | src/tla-safety.ts:386–429                                                                                            | crates/rtla-compiler/src/lib.rs:657–686                                                                                                                                                      | not the same type: bodyへ入るのはparameter/capture参照                                        |
| shared callのactual/capture格納            | src/tla-safety.ts:435–446                                                                                            | crates/rtla-compiler/src/lib.rs:543–553, crates/rtla-compiler/src/lib.rs:690–704                                                                                                             | not the same type: call配下へ置きcheckerがcaller frameを保存                                  |
| membership用途選択                         | src/tla-safety.ts:1245–1250, src/tla-safety.ts:435–440（問題の分岐src/tla-safety.ts:377→src/tla-safety.ts:618）      | crates/rtla-compiler/src/lib.rs:543–551, crates/rtla-compiler/src/lib.rs:1631–1640                                                                                                           | fixed-by-design: 用途付きshared経路へ統一。Rustだけ先にmembership lowerする差を除去           |
| LET宣言環境保存                            | src/tla-safety.ts:709–715                                                                                            | crates/rtla-compiler/src/lib.rs:992–1003                                                                                                                                                     | not the same type: 宣言位置を保存し、shared bodyではcapture参照化                             |
| fold明示actual                             | src/tla-safety.ts:675–695                                                                                            | crates/rtla-compiler/src/lib.rs:1414–1443                                                                                                                                                    | not the same type: v2は専用fold frameのactualをshared callへ格納。v1のbody代入は凍結          |
| target resolver                            | src/tla-safety.ts:1367–1414                                                                                          | crates/rtla-compiler/src/lib.rs:1106–1160                                                                                                                                                    | not the same type: explicit frameのarguments/capturesとparentを追う。静的解決の基礎として維持 |
| capture pruning / slot再番号               | src/tla-safety.ts:522–568                                                                                            | crates/rtla-compiler/src/lib.rs:738–772, crates/rtla-compiler/src/lib.rs:2418–2433, crates/rtla-compiler/src/lib.rs:2509–2529                                                                | not the same type: 同じ位置の配列と参照を対応し直す                                           |
| definition ID再番号                        | src/tla-safety.ts:2076–2094                                                                                          | crates/rtla-compiler/src/lib.rs:779–798, crates/rtla-compiler/src/lib.rs:2438–2454                                                                                                           | not the same type: call IDだけを対応し直す                                                    |
| identifier / oldのlocals参照               | src/tla-safety.ts:902–903, src/tla-safety.ts:942–943                                                                 | crates/rtla-compiler/src/lib.rs:1181–1182, crates/rtla-compiler/src/lib.rs:1232–1236                                                                                                         | not the same type: 参照読取りの末端。先行inlineの誤束縛を顕在化する                           |
| EXCEPT old導入                             | src/tla-safety.ts:870–883, src/tla-safety.ts:1000–1003                                                               | crates/rtla-compiler/src/lib.rs:1269–1278                                                                                                                                                    | not the same type: 新しいEXCEPT自身のoldを導入。actualのoldは別frame                          |
| binder / CHOOSE / comprehension導入        | src/tla-safety.ts:719–767, src/tla-safety.ts:1015–1038                                                               | crates/rtla-compiler/src/lib.rs:1012–1050, crates/rtla-compiler/src/lib.rs:1287–1318                                                                                                         | not the same type: 新しいbinder参照を導入。actualを移さない                                   |
| metadata用dummy formal/binder              | src/tla-safety.ts:1484–1486, src/tla-safety.ts:1547–1556                                                             | crates/rtla-compiler/src/lib.rs:1771–1776, crates/rtla-compiler/src/lib.rs:1828                                                                                                              | not the same type: 構造解析用FALSEで実actualの移送ではない                                    |
| level summaryのactual依存合成              | src/tla-safety.ts:271–276                                                                                            | crates/rtla-compiler/src/lib.rs:386–387                                                                                                                                                      | not the same type: metadataだけを合成                                                         |
| v1 domain / expression / membership inline | src/tla-safety.ts:821, src/tla-safety.ts:917, src/tla-safety.ts:1096, src/tla-safety.ts:1256, src/tla-safety.ts:1303 | crates/rtla-compiler/src/lib.rs:1085, crates/rtla-compiler/src/lib.rs:1204, crates/rtla-compiler/src/lib.rs:1471, crates/rtla-compiler/src/lib.rs:1634, crates/rtla-compiler/src/lib.rs:1683 | not the same type: v1凍結。対応する通常v2経路はshared callを維持                              |

行を特定したread-only sweepは独立した読み手にも依頼し、同じcaller-frame境界を適用するaction/temporal経路を設計へ含めた。これは別モデル系統のレビューや、未実装設計の検証成功を意味しない。

### step 3d-aのRed結果

新規TLCは8入力を一件ずつ実行した。EXCEPTの`= 4`だけが終了12・初期状態`x = 0`のSafe違反で、集計のない状態数/生成数をgoldenへ補っていない。他7入力は終了0、各1状態・生成2件。新しいexpectedだけへ追記し、既存のTLCログ/expectedとv1 baselineを保持した。

operator-sharing cohortは562件中540成功・22失敗、skip/TODO/cancelは0。既存474件と新規v1の32件、新規fixture検証8件は全て通過した。新規v2の48件は26成功・22失敗で、失敗は以下の6入力だけに属する。`native/Wasm check parity`という名前の失敗でも、応答一致自体は通過し、続くTLC oracle照合で落ちている。TSのmembershipはcompile拒否なのでTS生成IRのchecker呼出しへは進まない。

| 新規入力                                | TLC           | 現v1（両frontend）          | 現v2                                                         | Red件数 |
| --------------------------------------- | ------------- | --------------------------- | ------------------------------------------------------------ | ------: |
| sharing-generative-except               | 初期違反、x=0 | complete（既存誤束縛）      | 4経路ともcompleteを誤報告                                    |       4 |
| sharing-generative-except-control       | success       | 初期違反（既存誤束縛）      | 4経路とも偽の初期違反                                        |       4 |
| sharing-generative-binder               | success       | Invalid bound variable slot | 4経路ともInvalid safety variable read                        |       4 |
| sharing-generative-binder-control       | success       | complete                    | complete                                                     |       0 |
| sharing-generative-membership           | success       | operator Natのprotocol拒否  | TSは同拒否、Rust native/Wasmはcomplete。frontend構造も不一致 |       3 |
| sharing-generative-membership-alias     | success       | operator Natのprotocol拒否  | 同上                                                         |       3 |
| sharing-generative-binder-exact         | success       | Invalid bound variable slot | 4経路ともInvalid safety binding count                        |       4 |
| sharing-generative-binder-exact-control | success       | complete                    | complete                                                     |       0 |

v1の現状assertionはTLC適合の成功へ読み替えない。省略/明示v1の元の239＋17入力のbyte guard257件も通過した。既存UNCHANGED別名/parameter/nestedは全てGreenを維持する。設計と回帰入力を追加した段階であり、P0/P1の実装修正はこのstepでは行わない。

## step 3d-b: generative position resolution

生成的外殻全体へのactual IR代入を両frontendから除いた。source式とcaller/宣言環境をたどるtarget resolverを使い、生成的UNCHANGEDとENABLED内のsquare/angle subscriptだけを変数slot/tupleへ解決する。通常のUNCHANGEDとaction subscriptの値比較は維持する。必要なformal/captureを非再帰の依存順summaryで求め、要求bindingの初出順にtuple形とslotを保持したsignatureをkeyへ加える。同じsignatureを共有し、通常parameter/capture参照とcaller frameを保つ。aliasのactualへ戻る際はcallerの循環検出contextも復元する。

membership位置のformal/capture要求もsource graphで伝播し、IF/CASE/LETの結果へ用途を保持する。checkerのv2 membership descriptorは条件分岐の選択枝を同じframeで毎回解釈し、未選択枝を評価しない。値cacheは導入せず、v1と通常値の列挙、call課金と既存の全上限を維持する。

action/temporal外殻はAST展開を維持するが、通常actualをcallee localsへ貼らない。source actualをcaller環境でlowerし、bound/old/parameter/captureに依存しない形だけを通常位置で扱う。frame依存形は両frontendで`action arguments require lexical frames`のprotocol拒否とする。生成target用のactualはsource thunkから解決し、unused actualも要求された用途で構文検証する。これは任意のaction引数やaction template共有への対応ではない。

元の22 Redのgoldenやv1現状assertionを変更せず、複数slot・入れ子tuple・signature再利用・targetと通常parameterの併用、LET/captured target、非変数target拒否、callerを復元する入れ子alias、ENABLEDのsubscript、membershipのformal/alias/capture/分岐/binder domain、未使用action actualと型付き診断の20手書き対照を追加した。新しいTLC実行やgoldenの作り直しは行わない。これらの手書き対照は保存TLC由来の独立入力へ数えない。

最終結果はcohort582成功・失敗/skip/TODO/cancel 0。元の22 Red、既存474、新規v1 32、新規fixture検証8、新規v2 48を全て保持した。v1 baseline coverageとbyte guardは257成功で、1536経路の既存digest/拒否を維持した。最後のsource編集後のfmt/clippy/testはrtlc52、rtla-compiler59、idealis-core18成功、compiler-test59、safety-test795、check154、browser3、portable-test151（Chromium2を含む）、quint-test13が成功した。portable-buildとsupport-plan（7 crates/28 comparisons）、新規[step3d run](../runs/0048-operator-sharing-step3d/result.json)（239入力・237 matched・既知2 rejected）、evaluate（20固定seed・PBT12 records）とreplay-evaluation（人工故障5・schedule6）も成功した。人工故障の未検出は残し、正常対照やreplay失敗は0だった。

補助の239入力v2照合は235受理・4拒否。既知のUnicode拒否2件に加え、frame依存action引数を持つ2入力が今回の設計に従う型付き拒否になった。旧step3cのv1/v2意味同一scriptは最初の新規拒否で失敗し、そのログを保持する。別に条件を固定した全件診断ではTS/Rust構造または既知の拒否、native/Wasm compile/check各239応答が一致した。新規拒否2件を受理や意味結果一致へ数えず、既知Unicodeの診断文言差も保持する。残る237件（既知拒否2を含む）の意味結果はwork以外一致した。

レビューのEXCEPT再現は4入力×4経路の全16応答が初期invariant-violationになり、Nat/alias再現のv2は両frontendともcomplete・states1/generated2/work10、v1は従来のNat拒否だった。5000定義のinvariant/guard/prime/membership、変数target alias、formal付き生成target chainはTS/native/Wasmで正規化構造が一致し、cycleは型付き拒否だった。変数target aliasは静的解決後1定義、他は5001定義で、深いchainの実行成功を保証しない。

signature増加の手書き測定は異なるtarget 1/2/3/4/5に対しprogram全体の定義数2/3/4/5/6、TS bytes1229/1760/2424/3088/3778、Rust bytes1246/1794/2475/3156/3863。同じxを再度呼ぶ6 call目は両方6定義のまま、bytes3899/3984だった。差は既存のENABLED null subscriptの有無で、全点の正規化構造は一致した。sourceのtarget形によって定義数・bytesが増える事実を記録し、無条件の線形成長や評価仕事の削減とはしない。

任意の5仕様測定は一件ずつ、RSS 4000000 KB・300秒のwatchdogとNode heap2048 MiBで実行した。全20 commandのうち19終了、B Rust v1だけが195.3秒・RSS4009792 KBで打切り、成功や型付き拒否へ置き換えない。終了19件は受理2（Aのv1、deadlock・325状態・深さ9・work37395・61610 bytes）と型付き拒否17だった。v2は全10 commandが拒否で、checker成功や実プロジェクトの未知バグ発見はない。

| 入力 | v1 TS / Rust              | v2 TS / Rust              | v2 compile ms TS / Rust | v2 process peak MiB TS / Rust |
| ---- | ------------------------- | ------------------------- | ----------------------- | ----------------------------- |
| A    | deadlock / deadlock       | action frame拒否 / 同拒否 | 6.5 / 10.8              | 86.9 / 90.6                   |
| B    | nesting40拒否 / RSS打切り | compiled128拒否 / 同拒否  | 300.9 / 616.3           | 284.7 / 261.5                 |
| C    | nesting40拒否 / 同拒否    | action frame拒否 / 同拒否 | 49.8 / 52.1             | 131.7 / 108.4                 |
| D    | nesting40拒否 / 同拒否    | action frame拒否 / 同拒否 | 38.9 / 42.1             | 112.1 / 107.9                 |
| E    | nesting40拒否 / 同拒否    | action frame拒否 / 同拒否 | 76.4 / 96.9             | 140.5 / 147.6                 |

RSSは短命processを取り逃がすwatchdog標本でなく`time -l`のprocess最大。B TS v1は4802.8ms・1527.7MiB、E Rust v1は126389.3ms・2611.8MiBでnesting40拒否だった。v2拒否時のbytes/definitions/checkは未返却。Aのv2も今回はframe依存actionを拒否するため、前stepの受理を維持した、適用範囲が広がった、一般的に速くなったとは主張しない。

途中の追加membership対照2件は条件分岐後の列挙でboundedとなり、descriptor用途を選択枝へ保持して解消した。新規Rust unitのlocalScope不足1件とsupport-planのexcerpt-budgetも修正し、途中ログを保持する。support-planはmethod群を空行で区切って既存selectorを通常lowering群へ適用し、予算・selector・保存recordは変更していない。Jev APIキー未設定でsupport-syncは警告skip、offline support-checkはstale compiler manifestで失敗したまま。同期済みやCI全成功とは数えない。既存TLCログ/expected/v1 baseline・計画原本96ファイルのdigest、Git refsとindexは不変で、stage/commit/GitHubへの変更はない。

## frame-carrying action arguments

step 3e-aは設計とRed入力だけを追加する。step 3d-bの`action arguments require lexical frames`は健全性を守る暫定拒否だが、`Step(p) == x' = p` / `Next == \E p \in {1, 2}: Step(p)`まで拒否する。既存239入力の`upstream-channel`と`parallel-channels`もv1で受理されv2で拒否される。この縮小を受容するのではなく、actionとaction/temporal helperにもcaller frame付きactualを渡すことを次の実装の要件とする。action-template共有、v1変更、値cache、既存上限の緩和は行わない。

### 固定TLCの意味と比較する二案

TLC 1.7.4の固定revision `5a47802b5c391f59ecdd44117981f4ff8c0656ba`では、[getVal/getOpContext](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/SymbolNodeValueLookupProvider.java#L78-L113)がactualの式と呼出し前のcontextをLazyValueに保存してからformalを束縛する。calleeのformalやbinderをactualのcontextに混ぜない。[next-state application](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L945-L969)と[ENABLED application](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L2588-L2618)もこのcontextを使う。[@ lookup](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1440-L1443)はEXCEPT_ATをcontextから読み、[EXCEPT更新](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1871-L1897)がその更新対象のoldを新contextへ束縛する。actualの`@`はcallerのoldであり、callee自身のEXCEPTのoldではない。[LazyValueの要求時評価](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1529-L1542)でも保存contextと要求時のstateを分ける。以下のIR表現・課金はidealis側の設計で、TLCの内部IRやcache政策と同一という主張ではない。

| 案                                       | IR契約とvalidation                                                                                                                                                                                                                         | rtlcでの評価とArena::consume                                                                                                                                                        | 判断                                                                                                                                      |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| (a) call siteごとの非共有anonymous frame | v2だけへexpression、action step、temporal formulaのframe nodeを追加。arguments/capturesと埋込みbody、localScopeを持つ。actualはcaller scope、bodyは独立parameter/capture領域と空のlocalsで検証。未使用actualも検証する                     | 入口1 work、actual/capture要求ごと1 work。caller locals/frame/oldを保存し、current/next/primeは要求時に使う。actionはcontinuation付きbranch実行、temporalはformula全体のframeを保持 | 採用。既存shared expressionと同じ字句frame不変条件を、共有なしでactionに適用できる                                                        |
| (b) capture-avoiding substitution        | actualの全binderを宣言identityでalpha renameし、貼付位置ごとにfresh slotを割り当てる。callerのfree bound参照を保ち、calleeと区別したEXCEPT old領域を新設して明示再束縛する。parameter/capture参照も元frameを明示し、全actual検証を別に保持 | caller-old退避/復元とouter-parameter参照が必要。複製される要求に課金markerを残し、入口1 work・要求1 workを維持。unused actualやbinder domainを前倒し評価しない                      | 不採用。fresh binderだけではold/outer thunk/purposeを直せず、結局frame相当の参照機構が必要。複製・slot増大・short circuitの再検証も増える |

(b)のold再束縛は値へのeager変換で代用しない。outer EXCEPT導入時のoldを保存する別lexical領域と、actual内部のEXCEPTが導入するoldを分け、参照に領域identityを持たせる必要がある。挿入先のbinder offsetを足すだけの修正や、通常actualをbodyへ貼ってからframe依存なら拒否する方式は採らない。

### 採用案のwireと実行境界

これは次stepで実装するv2契約案であり、現在のdecoderが受理するという説明ではない。既存definition tableは`bodyKind: expression`だけを維持する。anonymous nodeにdefinition ID・宣言key・共有table entryを付けず、各source applicationをそのcall siteで一度lowerする。同じactionを二箇所から呼べばbodyも二つ生成する。body内の既存expression operatorだけは従来のcallを使える。再帰actionは拒否する。action bodyの共有templateや隠れたinternerは追加しない。

- expression: `{op: "frame", arguments: [...], captures: [...], parameterCount: n, captureCount: m, localScope: k, body: expression}`。
- action step: `{kind: "frame", arguments: [...], captures: [...], parameterCount: n, captureCount: m, localScope: k, branches: [...]}`。branchesは既存Branch構造。GuardやAssignごとのexpression wrapperだけではcallee binderをまとめられないため、action body全体を包む。
- temporal formula: `{kind: "frame", arguments: [...], captures: [...], parameterCount: n, captureCount: m, localScope: k, formula: temporalFormula}`。formula内のquantifierとatomを同じ境界に置く。boxed/angleのaction部分も必要ならaction frameを持つ。

countsは必須で配列長と一致し、parameter/captureは既存slot参照を再利用する。callerのbinder/formal/oldは明示captureでbodyから参照する。LETの宣言bindingは宣言identityによりcapture順を固定し、call siteでその宣言環境を参照する式をcapturesへ投影する。arguments/capturesは共にcaller lexical frameに保存するthunkであり、callee localsへIRそのものを置かない。calleeのlocalsは空、oldはNoneから始め、EXCEPTで導入したoldだけが直接見える。caller `@`はcaptureまたはargumentを経由する。anonymous frameの内部からshared expressionへ渡るcaptureもこのparameter/capture参照で、markerや負slotをwireへ漏らさない。

rtlcは既存`Argument::new/evaluate`と同じ契約で、式とcallerのimmutable locals、親frame、oldを保持する。actualに含むbinderは要求のたびcaller localsの続きへ導入し、callee側のbinderとは別領域にする。shadowingは名前ではなく宣言identityで解決する。引数・結果の値cacheは持たず、unused actualは実行しない。current/next/primeは保存せず、prime下の要求とENABLEDのfresh next候補を毎回の動的contextから渡す。membership用途もthunkを通して保持し、Natやrangeを通常値へ列挙してから貼り直さない。

`visit_branch`はframe stepでcaller contextを保存し、callee contextで全内側branchを訪問する。各inner branch完了時、同じnext候補とarenaを使って外側の残りstepsへcontinuationを呼ぶ。inner binder/old/frameは外へ漏らさず、内側assignのnextはcontinuation中に維持し、backtrack時には従来どおり復元する。outer binder `p`の値はthunkが参照するsnapshotとして生存させる。callee内のBind slotは0から始める。ENABLEDも同じexecutorを使い、最初の有効candidateで短絡し、fresh nextを外へ漏らさない。

temporal helperはAST分類時にactualを評価せず、lower時にformula frameを保存する。temporal quantifierのbindingとhelperのbindingを別scopeにし、atom/state/action評価時にも保存lexical frameと要求時stateを組み合わせる。探索後まで保持するformula/candidateは所有されたimmutable snapshotを使い、branch stackへの借用を保存しない。frame作成は値評価ではなく、formula bodyを評価するたびに入口1 workを課す。atom結果やENABLED結果をmemoizeして入口・要求の費用を飛ばさない。

validationは全arguments/capturesをcallerのphase/purpose/slot容量で先に検査し、bodyは新しいframe signatureで別検査する。全branch、未選択枝、未使用actualも構文検査するが評価しない。bodyのparameter/capture要求metadataをactualへ合成し、phase・constant/context-free・membership・next読取りの検査を維持する。definition DAGの探索はanonymous body内のshared call edgeも走査し、cycleを隠さない。pruning、capture再番号、call ID再番号、serialized-depth検査は新scopeを認識する。v1は新nodeを拒否し、既定/明示v1のbytes・診断・意味結果を変えない。request envelope/ABI versionも1を維持する。

`Arena::consume(1)`はanonymous expression/step/formulaへ入るたび、parameter/captureを要求するたびにそれぞれ課す。既存expression評価、branch tick、binder列挙、ENABLED候補、temporal仕事の費用は別に維持する。直接expressionのanonymous `Id(Id(TRUE))`は従来callと同じ4 workを基準とし、maxWork 3/4/5はbounded-work/成功/成功。actionでは既存step tickにframe入口1を追加し、2回使うactualには要求2回分を加える。unused actualに要求費用を課さない。枯渇はbounded/reason:workで、拒否・例外・上限超過をcompleteにしない。

localScope40、definition内式深さ48、値深さ64、decoder128、動的入れ子128、branch steps256・branch数4096、frontend nesting40を維持する。frameでstep/branchを細分して上限を迂回しないよう、埋込みbodyも一つのsource expansionに対して再帰的に集計する。frame自身も深さへ数える。表現増分・検証時間・評価workは別に測り、action body共有や指数的仕事の削減とは数えない。

### 生成targetと値参照の分離

3d-bの`(source actual, caller/declaring lexical environment)` resolverを維持する。UNCHANGEDの生成target、ENABLED内部の生成的UNCHANGEDとsquare/angle subscript、SPECIFICATION/fairnessの変数subscriptがformal/captureへ達したときだけ、匿名frameのsource thunkを呼出し前環境へ戻ってたどる。state variableならslot、tupleなら順序・入れ子を保持したdescriptorを返す。alias/formal/captureの連鎖は宣言identityと元callerの循環contextで追う。anonymous frameには共有keyがないが、内側のshared expressionは3d-bと同じtarget signatureでspecializeする。

生成stepのAssign/Choose先とENABLEDのsubscriptは、この静的descriptorのslot/tupleを直接使う。実行時のparameter値を変数番号と解釈しない。binder、`@`、定数、計算式は生成targetとして引き続きtyped protocol拒否にする。一方、同じformalをRHS/guard/通常UNCHANGED/通常square・angle subscriptで読む場合はparameter thunkのまま値評価する。`Step(v,p) == UNCHANGED v /\ x' = p`へ`(y, callerBinder)`を渡すと、vの生成位置はyへ解決し、pはframe付きで評価する。pまで静的変数targetへ解決しようとしない。

### 両frontendのsame-type sweep

以下はstep 3d-b後の実装を読んだ行番号。dispositionは後続設計での扱いであり、今回の実装変更ではない。refusal文字列だけでなく、安全と判定して返すactual IR、Rustの`@` clone、呼出し側も含める。

| 位置                                       | TS (`src/tla-safety.ts`)                                                              | Rust (`crates/rtla-compiler/src/lib.rs`)                         | disposition                                                                     |
| ------------------------------------------ | ------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| helperの宣言capture投影・actual marker     | call:1236–1318、v2:1267–1294                                                          | call:1706–1795、v2:1737–1778                                     | anonymous frameのparameter/captureへ置換。callee localsへの実actual投影をしない |
| 通常actualの要求・frame拒否                | identifier:1578–1600（1590/1597）                                                     | inline_value:432–469（450/461）、expression:2058–2060            | caller thunk要求へ統一。frame依存拒否と安全判定後のIR貼付を廃止                 |
| helper bindingをshared expressionがcapture | sharedCall:1080–1099（1091/1096）                                                     | shared capture:1481–1504 → inline_value:1489–1493                | 新frameのparameter/captureをshared callへ渡す                                   |
| membership actual                          | membershipSet:1928–1950（1942/1947）                                                  | membership:2502–2504 → inline_value(true)                        | caller frameとmembership用途を同時に保持                                        |
| helper captureの`@`                        | ExceptAt:1642–1646（1645）                                                            | ExceptAt:2112–2116のlocals clone                                 | caller oldを明示capture。Rustのinline_valueを通らないcloneも対象                |
| generative action application              | action:2369–2395 → call                                                               | action:2802–2822 → call                                          | branches全体をcall site固有frameにlower                                         |
| ENABLED入口                                | expression:1633–1640 → action                                                         | expression:2106–2110 → action                                    | fresh next/短絡を維持して同じframe executorを利用                               |
| boxed-action判定                           | isBoxedAction:2606–2618 → call                                                        | boxed_action:2986–2996 → call                                    | 値評価せずframe計画を保持。AST判定でactual IRを貼らない                         |
| temporal helper                            | temporal:2621–2635 → call                                                             | temporal:3022–3033 → call                                        | formula全体にframe境界を保存                                                    |
| root / SPECIFICATION alias                 | operator:2588–2589、parts:2742–2749                                                   | operator:3176、parts:3110–3112                                   | 現rootはactualなし。capture付きaliasにも同じ契約を適用                          |
| 通常shared expression                      | sharedCall:972–1101、1616/1799                                                        | shared_call:1289以降、2082/2349                                  | 維持。definition tableはexpressionだけ                                          |
| fold明示actual                             | 1344–1367、v2:1356–1360                                                               | v2:2289–2307、v1:2309–2323                                       | v2専用accumulator/element frameは維持。helper由来captureは新frame参照へ         |
| v1のactual/capture貼付                     | call:1268–1287、1495/1617/1800/2008/2057、fold:1361–1367                              | call:1797–1828、1961/2084/2351/2548/2597、fold:2309–2323         | v1凍結。Rust1798–1803のv2条件は1795のreturn後で到達不能                         |
| capture pruning / ID再番号                 | compactCaptures:1176–1234                                                             | finalize:1547–1619、remap:3369以降                               | scopeをまたぐcapture slotの誤集計を防ぎ、anonymous bodyのshared callを走査      |
| static target                              | resolveTarget:350以降、targetRequirements:499以降、variableReads:2112–2172、2348/2517 | call_targets:1313–1357、variable_reads:1980–2042、resolve_target | source環境付きresolverを維持。通常actualを貼る処理に戻さない                    |

rtlcでは`ir.rs`のExpression/Step/TemporalFormula、v1新node拒否、`safety_definitions.rs`のbranch/metadata/definition-order traversal、`safety_expressions.rs`のArgument/CallFrame・visit_branch・ENABLED、temporal evaluatorとdecoderが後続対象となる。Guard/RHSだけを包む案ではcallerとcallee binderの境界を表せず、既存Step四種だけでは不足する。独立したread-only sweepも同じ境界を確認した。指定reviewer roleが利用不可のため同等のread-only制約で通常roleを使用しており、別モデル系統の証拠とはしない。

### Red入力の事前条件

結果を見る前に次の7入力を固定した。存在量化の1変数版と2変数版、binder-bearing actual、callerとcalleeで同名pを導入するshadow版、外側EXCEPTの`@`をactionへ渡しcalleeに別EXCEPTがある再現と正常対照、binder下のENABLED action callを独立fixtureとする。EXCEPT比較4/9の両方を残し、v1の既存誤束縛も現状assertionとして固定する。v1の誤completeをTLC適合とは数えない。

新規tiny fixtureだけを従来capture手順（固定TLC 1.7.4、timeout60、heap256m、workers1）で一件ずつ実行する。goldenは新規expectedへTLCログからだけ保存する。v2はTS/Rustの意味結果・正規化IR構造・native/Wasm完全応答が一致し、受理することを要求する。v1は現状結果を別に要求する。既存239入力もv2で照合し、既知Unicode2拒否は保持するが、upstream-channel/parallel-channelsは拒否を許容せずv1と同じ意味結果を要求する。workだけは意味比較から除く。既存plan/log/expected/v1 digestは編集せず、src/・crates/も変更しない。意図したRedは新規7入力のv2 assertionと既存2入力だけである。

### step 3e-aのRed結果

新規TLCは7入力を一件ずつ採取した。EXCEPT再現は初期Safe違反、他6件はsuccess。状態数/生成数はexists・two-variables・shadowが各3/7、binder-actualが2/3、enabledとexcept-controlが各1/2。初期違反ログにない集計は補わず、新しい[expected-step3e-a.json](../fixtures/tlc-operator-sharing/expected-step3e-a.json)へ保存した。v1現状は独立した[v1-current-step3e-a.json](../fixtures/tlc-operator-sharing/v1-current-step3e-a.json)へ固定した。

cohort最終は898件中854成功・44失敗、skip/TODO/cancelは0。元の582テストの名前集合が全て成功側に含まれることを確認した。新規v1の28件とfixture検証7件は全て成功。新規v2は7入力それぞれTS/TLC、Rust/TLC、frontend parity、native/Wasm compile parity、native/Wasm check parity、TS native/Wasm check parityの6件、計42件がRedである。いずれも`Unsupported TLA input: action arguments require lexical frames`による受理失敗で、両者の共通拒否をparity成功とは数えない。TS生成IRのcheck parityはfrontend拒否でcheckerへ到達しない。

既存239入力の新しいv2検査は237成功・2失敗。既知Unicode2件は従来のtyped refusalを確認し、それ以外はcompile受理・TS/Rust構造・native/Wasm応答・v1意味結果を要求した。失敗はupstream-channelと保存入力parallel-channelsだけで、同じaction-frame拒否による。v1のcompleteとbounded/statesをそれぞれ保持すべきであり、拒否を同じ意味結果に読み替えない。

最初のcohortは817成功・81失敗だった。新しい既存入力用helperが、意図的な実行時エラー37件にもkind=safetyを要求したためで、compile受理を維持したまま、エラーを含むv1の正確な意味結果との比較へ修正した。fixture・golden・snapshot・予算・実装は変えていない。初回ログと全37件のテスト名を残し、最終44 Redだけになることを再実行した。

just checkは整形後154成功、browserは実Chromium3成功、evaluate/replay-evaluationは正常対照と再現の失敗0で完了した。support-planは7 crates/28比較を生成し、support-syncはJevキー未設定の警告skip。同期済みとは数えず、stageもしない。設計とRed入力の追加が今回の成果であり、frame nodeの実装、任意のaction/temporal受理、全TLA+互換の証拠ではない。

## step 3e-b 前提レビューの反映（実装前）

P1-1のLET定義本体に、その本体内のEXCEPTで束縛されない自由な`@`がある形は、v2のtyped protocol拒否とする。引数あり・なしのレビュー入力を両方拒否対照へ追加し、宣言時oldと適用時oldのどちらかをcaptureする対応は今回実装しない。actual内のcaller `@`を保持する既存設計は維持する。

P2-1としてactualの値でENABLEDが変わる独立入力を追加する。P2-2としてframe依存actualを持つ`Temporal(p) == [](p)`と、caller binder・状態変数を同時に参照するactualをcalleeのprime下で要求するactionを追加する。新規tiny入力だけを固定TLC 1.7.4で一件ずつ採取し、期待値はログから作る。従来のgenerative-action/temporalのframe不足拒否assertionは、新設計の受理とTLC意味結果の要求へ変更する。

成功条件は3e-aの44 Red解消、追加入力の両frontend構造・native/Wasm完全応答・TLC結果の一致、および既存回帰・v1 bytes・上限の維持とする。既存goldenやbaselineは変更せず、新規記録を追加する。

### step 3e-b 途中の観測

初回cohortは943件中932成功・11失敗だった。3e-aの44 Redは全て成功へ移った。追加ENABLEDの6失敗は、レビュー入力の`<=>`が従来の両frontendで未対応だったためで、元入力・TLCログを変更せずv2限定のBoolean等価loweringを追加する。`eq(not(left), not(right))`により両辺のBoolean型検査と左から一度ずつの評価を維持し、v1は変更しない。

残る5失敗は同じtemporal helper入力のTLC意味比較だった。固定TLCがtemporal違反を報告する一方、実装はcompleteを返した。helperを通さない`[]TRUE`と直接の量化式はTLCでsuccess、状態依存actualのhelperは未対応診断だった。原入力・ログ・goldenを保持し、差を合格へ読み替えたり、未確認の特殊処理で違反を模倣したりしない。

固定ソースの読取では、直接のboxは[SpecProcessorのinvariant分類](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/SpecProcessor.java#L1349-L1392)へ入り、引数ありhelperはtemporal経路へ進む。[Livenessのhelper展開](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/liveness/Liveness.java#L105-L134)はactualをEmpty状態で評価する。constant TRUEのboxを否定した`<>FALSE`が残り、[tableau粒子の整合性検査](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/liveness/TBPar.java#L243-L265)と状態predicate抽出がBoolean FALSEを検査しない経路が、観測された違反を説明する。これは固定revisionの読取による原因推論で、内部tableauを計装して確認した結果や他revisionへの一般化ではない。状態依存actualの未対応はEmpty評価失敗後のlevel拒否という別経路である。lexical frameの意味をこの違反へ合わせる特殊処理は追加しない。

### step 3e-b 実装と必須検証

両frontendとrtlcへ非共有anonymous expression/action/temporal frameを追加した。引数/captureはcallerのimmutable locals・old・親frameを保持し、state/next/primeは要求時に使う。actionの内側branchから同じnext候補でcaller側continuationへ戻り、backtrackで代入を復元する。temporal frameは所有snapshotを使い、空量化を含むframe入口と引数要求を課金する。未使用actual・未選択branchの検証、phase metadata合成、membership用途、DAG、capture剪定、既存の深さ・step・branch上限を維持した。手書きRust IRの5000段frameは検証前走査を反復処理とし、深さ拒否の回帰を追加した。

最終cohortは953件中948成功・5失敗、skip/TODO/cancelは0。3e-aの44 Redは全て成功した。従来582のうち要求どおり変更した旧拒否2本を除く580件、3e-aで成功していた854の同2本を除く852件は全て成功を維持する。旧generative action拒否はTLC結果への受理検査として成功した。残る5失敗は全て同じtemporal helperのTLC意味比較で、TS/Rustおよびnative/Wasmはcomplete、保存TLCはliveness-violationである。共通completeやcompile parityをTLC適合へ数えず、ゼロ失敗の目標は未達とする。

最後の実装・回帰修正後、rtlc62、rtla-compiler67、idealis-core18のcargo tests、各crateのfmt/clippy、compiler-test67、check154、browser3、portable-test（Rust・TS・Chromium）、quint-test13、v1 identity257、decoder10が成功した。5000定義のinvariant/guard/prime/membershipはTS/native/Wasmのcompile-only構造を維持する。safety-testは1166中1161成功・同じ5失敗である。新規[step3e run](../runs/0048-operator-sharing-step3e/result.json)と回帰ハーネス修正後の[delivery run](../runs/0048-operator-sharing-step3e-delivery/result.json)は各239入力・237受理・既知2拒否で、evaluate/replay-evaluationも完了した。人工故障の検出を実プロジェクトの未知バグ発見には数えない。

初回の追加v1 equivalenceテスト2本は、従来のTS/Rust診断差まで同一にする誤った期待値だった。frontendやbaselineを変えず、TSの`expression Apply <=>`とRustの`operator <=>`を個別に検査するよう訂正した。途中の7失敗ログを残し、再実行で同2件の成功とtemporal5失敗だけの集合を確認した。

指定の3再現scriptを別scratchへコピーして再実行した。EXCEPT4入力×4経路は全16件が初期Safe違反、Nat/aliasのv2は両frontendでcomplete・states1/generated2/work10、v1はNat拒否を維持した。存在量化actionの1変数・2変数はv1/v2ともcompile受理へ回復した。

support-planは7 crates/28比較を生成し、support-syncはJevキー未設定で警告skip。offline support-checkはrtla検査後、rtla-compilerのstale manifestで終了3となった。保存recordやdigestを手編集せず、同期済み・CI成功とは扱わない。全変更は未コミットで、stage/unstage・git履歴/refs・GitHubへの書込みは行わない。

### step 3e-b 5仕様の追加測定

同じrun scriptのコピーで、20 commandを一つずつ既存watchdog（RSS4000000 KB・300秒、Node heap2048 MiB）配下に実行した。19 commandが終了し、B Rust v1だけが300.3秒でtimeoutとなった。4 commandがcompile受理（Aのv1/v2×両frontend）、15 commandがtyped protocol拒否。打切りを拒否・成功に置き換えない。

| 入力 | v1 TS / Rust            | v2 TS / Rust                | v2 compile ms TS / Rust | v2 check                                |
| ---- | ----------------------- | --------------------------- | ----------------------- | --------------------------------------- |
| A    | 受理 / 受理             | 受理 / 受理                 | 11.7 / 24.9             | 両方deadlock・325状態・深さ9・work97078 |
| B    | nesting40拒否 / timeout | compiled128拒否 / 同拒否    | 391.2 / 738.0           | 未実行                                  |
| C    | nesting40拒否 / 同拒否  | 未対応operator拒否 / 同分類 | 70.7 / 78.5             | 未実行                                  |
| D    | nesting40拒否 / 同拒否  | nesting40拒否 / 同拒否      | 63.7 / 114.3            | 未実行                                  |
| E    | nesting40拒否 / 同拒否  | nesting40拒否 / 同拒否      | 117.2 / 172.7           | 未実行                                  |

Aのv2は両frontendとも31741 bytes・20定義で、v1は61610 bytes。Aの違反は保存済みv1のdeadlockと同じで、未知バグ発見には数えない。元scriptどおり今回のv1はcompileのみで、v2のcheck予算はmaxDepth64 / maxStates10000 / maxWork10000000。Cはprotocol分類が一致するが診断の表現に従来のfrontend差があり、完全な拒否応答一致へ数えない。Bはv1 TS4534.3ms、Rustは完了結果なし。E Rust v1は260547.0msでnesting40拒否だった。

compile時間は各processのWasmロード後に計測する既存scriptの区間で、cold start全体や反復した性能比較ではない。watchdogの1秒RSS標本は短命processの最大を取り逃がしたため、process peak RSSやメモリ改善の根拠には使わない。一般的な速度改善、全5仕様の受理、全TLA+互換を主張しない。

## step 3e-c: 固定TLCのtemporal helper乖離を記録

step 3e-bの5失敗を再実行で確認した後、`sharing-action-frame-temporal`だけを[記録済みTLC乖離](../fixtures/tlc-operator-sharing/divergences.json)として扱う条件へ変更する。この性質は`Temporal((\A b \in {0}: b = 0))`、すなわち`[]TRUE`で全behaviourに成立し、固定TLCの違反を意味論のoracleには使えない。保存ログ・expected・v1 digest・原入力をそのまま保持し、実装にこの入力の特殊処理は加えない。他のfixtureの期待値は変えない。

機械可読記録には固定revision、TLCのexit13・temporal違反・初期x=0からstuttering・状態数1/生成数2・ログdigestと、意味上のcomplete・状態数1/生成数2・深さ0・work11を分けて保存する。直接の2対照のworkは5/7であり、意味結果の一致と費用を混同しない。固定ソースでは直接boxのinvariant分類とhelperのliveness経路が異なり、否定後の`<>FALSE`のFALSE粒子を整合・promise充足と扱う経路が観測を説明する。公開固定revisionリンクを記録に含めるが、原因の説明はソース読取による推論であり、内部tableauの計装実行や他revisionへの一般化ではない。

既存5テストの対象fixtureだけを記録へ照合し、TS/Rustの意味結果・正規化構造・native/Wasm完全応答を要求する。専用テストで記録自体のdigestと唯一の登録名を固定し、直接の`sharing-temporal-true-step3e-b` / `sharing-temporal-forall-step3e-b`も従来のTLC success goldenと同じ意味結果へ照合する。正常fixtureへの誤登録、ログまたは記録された違反の変更をrunnerが拒否する負対照を追加する。入力を除外・skipせず、記録済み乖離の成功をTLC一致や未知バグ発見へ数えない。`sharing-action-frame-temporal-state`は従来のTLC未対応（exit255）で、一致・乖離のいずれにも含めない。

検査条件は変更前に固定する。3 crateのfmt/clippy/all-targets locked tests、compiler-test、safety-test、check、browser、portable-build/test、quint-test、support-plan、operator-sharing cohort、frame decoder、v1 identity subset、evaluate/replay-evaluationを最後の編集後に実行する。support-checkのstale manifestはmaintainerによるJev同期を要する失敗として報告し、保存support record/digestを手編集しない。新しいTLC採取、mutation、上限変更、stage/unstage、Git履歴・refsやGitHubへの書込みは行わない。

## step 3e-d: 関数定義域を含むv2解析

第三回レビューの結果を受け、Rustの定義解析にTSの`bodyOf()`と等価な正規化を追加する。関数定義はheadのboundsとbodyをまとめた`FunctionConstruction`として扱い、target要求・membership要求・LET自由`@`の拒否・nested LETの自由`@`走査へ適用する。既に構築全体を扱うIR生成と、Operatorだけを対象にするtarget resolver・fold・generative分類は維持する。TSの同型走査は既にboundsを含むため変更しない。

回帰条件は修正結果を見る前のレビュー記録から固定する。自由`@`が関数定義域にある使用・未使用・nested LET・ENABLED経由、および値部分にある対照を、head形式と明示的な関数構築形式で拒否する。生成target要求のBとmembership要求のCも両形式で受理し、maxDepth8/maxStates100/maxWork10000でそれぞれcomplete・states1/generated2/depth0/work13とwork11を要求する。TS/Rustの正規化構造または同じtyped protocol拒否、native/Wasmの完全応答を照合する。期待値を修正後Rustの出力から採取しない。保存済みTLC success対照との意味比較は新規入力のTLC直接実行と区別し、既存ログ・expected・v1 baseline・乖離記録は変更しない。

action/temporalの通常actualをframe依存なら拒否すると記すREADMEとfrontend文書の旧説明も、caller lexical frame付きanonymous frameによる保持へ更新する。生成targetだけを静的に解決し、LET自由`@`のtyped refusalは関数定義域にも適用する。過去stepの拒否記録は履歴として保持し、support recordやdigestを手編集しない。

修正前の14件は8成功・6失敗で、失敗はhead形式の自由`@`4種とB/Cだけだった。修正後はoperator-sharing cohort970件、frame decoder10件、v1 identity257件、safety-test1183件が全て成功し、失敗・skip・TODO・cancelは0だった。3 crateのfmt/clippyとcargo tests（rtlc62・rtla-compiler69・idealis-core18）、compiler-test69、browser3、portable-test171（内Chromium2）、quint-test13、portable-build、support-plan（7 crates/28比較）、evaluateとreplay-evaluationも成功した。support-checkはrtlaのoffline3/3後、既知のrtla-compiler stale manifestで終了3となり、同期済み・CI成功へ読み替えない。最初のcheckはこの節の余分な空行だけで失敗したため整形し、再実行で154件成功した。保存support record/digestは編集せず、Git履歴・refs・indexとGitHubへの書込みを行わない。

レビューのprobe全11本をコピーから再実行し、全commandが終了0、timeout・stderrは0だった。自由`@`の5入力×4経路は同じprotocol拒否、B/Cと明示構築対照の4入力×4経路はレビュー記録のcomplete・work13/11へ一致した。既存の反例・membership・target alias・深さ5000のcompile-only対照・frame深さ対照は保存レビュー結果を維持した。探索用probeに残る古いwantとの不一致は既知制限として区別し、新規テスト失敗やTLC一致へ読み替えない。新規TLCの採取、上限変更、mutation、background processの残留はない。

## step 3f: systematic analysis parity

step 3e-d後にもCHOOSEのdomain、`\notin`、CASEのcondition、nativeのdefinition countで差が残った。個別の分岐追加を続けず、v2解析が使う子の列挙と位置の意味を固定する。以下は設計であり、このstepでは実装・テスト・保存結果を変更しない。

2026-11-17 **3f-a2 revision**: 前提レビューのC1を受けaction展開集計とPROPERTY分類をinventory・単一列挙の利用対象へ追加した。C2のSubSeq第3引数漏れを解消するため固定arity・同種可変長・ternaryの位置表からcorpus件数と時間予算を再計算した。C3は差の保存だけでは同一受理fragmentの契約を満たさないため、TSの明示v2に有限CHOOSEを追加し、caller binder/old・actual・無解/空domainとTLC対照をscopeへ含める。これら以外の設計・既存境界は維持する。

2026-11-17 **3f-a3 revision**: C4の`~>`両引数漏れを、両parser/lexer/ASTと式・action・temporal loweringの機械inventoryから修正する。共有aliasも全位置へ展開し、`:=`等の非共有受理とparser-only構文を名前付きで分離した。PROPERTYの左右4probe・LET自由old拒否・到達assertionを固定し、source 3168入力、補完込み上限4864入力、180秒の必要速度を再導出した。既存4096枠へ位置を削って合わせず、未実測の時間予算とcoverageの成立を区別する。

### 解析のinventory

行番号はこの設計時点。TS欄は`src/tla-safety.ts`、Rust欄は`crates/rtla-compiler/src/lib.rs`を指す。source ASTと生成IRは別の木であり、IR解析をsourceの列挙器へそのまま接続しない。

| 解析                            | TS: function / file:line                                                                                                                                               | Rust: function / file:line                                                                                                                                                                                                                                                                                                                          | 現在の子の列挙                                                                                                                |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| 定義の正規化・sourceの子        | `bodyOf (src/tla-safety.ts:219)`                                                                                                                                       | `definition_body (crates/rtla-compiler/src/lib.rs:46)`, `source_children (crates/rtla-compiler/src/lib.rs:140)`                                                                                                                                                                                                                                     | headのFunction bounds+bodyを正規化。Rustの子列挙は手選択matchとwildcardで、CHOOSE.setを欠く                                   |
| free-`@` / free-old             | `hasFreeOld (src/tla-safety.ts:1521)`                                                                                                                                  | `free_old (crates/rtla-compiler/src/lib.rs:3517)`                                                                                                                                                                                                                                                                                                   | TSはObject.entries反射＋EXCEPT特例。RustはEXCEPT/LET/binder手選択＋source_children                                            |
| target要求・call target対応     | `targetRequirements (src/tla-safety.ts:606)`（compose (src/tla-safety.ts:719)）, `sharedCall (src/tla-safety.ts:1096)`                                                 | `requirements (crates/rtla-compiler/src/lib.rs:616)`, `call_targets (crates/rtla-compiler/src/lib.rs:1337)`                                                                                                                                                                                                                                         | scope/call/binderを手選択、残りはTS反射/Rust wildcard。要求bindingからformal actual/captureへの対応は手選択                   |
| 静的target解決・IR読取り        | `resolveTarget (src/tla-safety.ts:457)`, `variableReads (src/tla-safety.ts:2272)`, `unchangedSteps (src/tla-safety.ts:2342)`                                           | `resolve_target (crates/rtla-compiler/src/lib.rs:475)`, `variable_reads (crates/rtla-compiler/src/lib.rs:2025)`, `unchanged (crates/rtla-compiler/src/lib.rs:305)`                                                                                                                                                                                  | sourceのtuple/alias/LET、IRのread/tuple/call/frame等だけを手選択し、他は拒否                                                  |
| membership demands              | `membershipRequirements (src/tla-safety.ts:850)`                                                                                                                       | `membership_demands (crates/rtla-compiler/src/lib.rs:888)`                                                                                                                                                                                                                                                                                          | scope/binder/callと用途を手選択、残りはTS反射/Rust wildcard。Rustはinだけ、TSはin/notin                                       |
| generative-position分類         | `generativeCall (src/tla-safety.ts:2349)`                                                                                                                              | `generative_call (crates/rtla-compiler/src/lib.rs:2678)`                                                                                                                                                                                                                                                                                            | actionを伝える枝だけを手選択。Rustはwildcard=false                                                                            |
| action展開の経路数・最大step数  | `limited (src/tla-safety.ts:2244)`内の`size (src/tla-safety.ts:2247)`                                                                                                  | `action_dimensions (crates/rtla-compiler/src/lib.rs:270)`, `limited (crates/rtla-compiler/src/lib.rs:293)`                                                                                                                                                                                                                                          | branch.stepsからaction frame.branchesへ独自再帰。frame bodyの経路数を乗算・step数を加算し、actual/captureや値式内部は数えない |
| level summary・定義DAG          | `summary (src/tla-safety.ts:258)`, `level (src/tla-safety.ts:325)`, `prepareLevels (src/tla-safety.ts:1221)`                                                           | `level_summary (crates/rtla-compiler/src/lib.rs:1178)`, `level (crates/rtla-compiler/src/lib.rs:1235)`, `prepare_levels (crates/rtla-compiler/src/lib.rs:1241)`, `definition_order (crates/rtla-compiler/src/lib.rs:1634)`                                                                                                                          | call/frame/read/prime/enabled特例＋IRのObject.values/JSON反射。DAGは全callを集め反復処理                                      |
| capture収集・剪定・localScope   | `localScopeOf (src/tla-safety.ts:369)`, `compactAnonymous (src/tla-safety.ts:396)`, `compactCaptures (src/tla-safety.ts:1261)`                                         | `finish_shared (crates/rtla-compiler/src/lib.rs:1544)`, `compact_anonymous (crates/rtla-compiler/src/lib.rs:3459)`, `anonymous_frame (crates/rtla-compiler/src/lib.rs:3493)`, `capture_slots (crates/rtla-compiler/src/lib.rs:3565)`, `remap_captures (crates/rtla-compiler/src/lib.rs:3587)`, `local_scope (crates/rtla-compiler/src/lib.rs:3634)` | IR反射走査。nested frameのbodyをscope境界として遮断。初期capture候補は宣言環境のprojection                                    |
| call収集・再番号付け            | `visit (src/tla-safety.ts:332)`, `visitScope (src/tla-safety.ts:348)`, `compactCaptures (src/tla-safety.ts:1261)`, 最終ID remap (src/tla-safety.ts:3139)               | `call_ids (crates/rtla-compiler/src/lib.rs:3668)`, `remap_calls (crates/rtla-compiler/src/lib.rs:3614)`, `trim_call_captures (crates/rtla-compiler/src/lib.rs:3688)`                                                                                                                                                                                | IR全フィールドの反射走査。scope走査だけframe bodyを遮断                                                                       |
| prime・temporal・compiled depth | `prime (src/tla-safety.ts:1722)`, `validatePrime (src/tla-safety.ts:1316)`, `validateTemporalActions (src/tla-safety.ts:2946)`, 無名depth走査 (src/tla-safety.ts:3151) | `prime (crates/rtla-compiler/src/lib.rs:1248)`, `check_primes (crates/rtla-compiler/src/lib.rs:1672)`, `validate_temporal_scoped (crates/rtla-compiler/src/lib.rs:1264)`, `finish_shared (crates/rtla-compiler/src/lib.rs:1614)`                                                                                                                    | levelを参照し、IRを反射またはkind手選択で走査                                                                                 |
| boxed action・SPECIFICATION分解 | `isBoxedAction (src/tla-safety.ts:2791)`, `parts (src/tla-safety.ts:2986)`（reads (src/tla-safety.ts:3002)）                                                           | `boxed_action (crates/rtla-compiler/src/lib.rs:3044)`, `parts (crates/rtla-compiler/src/lib.rs:3188)`, `read_slots (crates/rtla-compiler/src/lib.rs:3385)`                                                                                                                                                                                          | alias/actual、連言、box/fairness、read/tupleを手選択                                                                          |
| PROPERTYのatom/always分類       | `formulaAtom (src/tla-safety.ts:2930)`, `formulaAlways (src/tla-safety.ts:2938)`                                                                                       | `atom_expression (crates/rtla-compiler/src/lib.rs:3441)`, `always_expression (crates/rtla-compiler/src/lib.rs:3450)`                                                                                                                                                                                                                                | temporal frame.formulaを潜り、atomまたはalways直下のatomを分類。frameの再構成を伴う                                           |

### 一つの列挙入口とrole

TSに`analysisChildren`、Rustに`analysis_children`を一つずつ置く。入力はsource / typed IRを区別するprivateなnode viewとし、Definition、bound/domain、EXCEPT path、step/branch、temporal formula、programも同じ入口で展開する。各edgeは子、安定したposition path、role、scope効果、action/membershipのforward規則と既存の深さ集計規則を持つ。Rustは型付きenumの全variantと式を含む全fieldをwildcard/`..`なしでmatchする。現在JSONのIRは各定義/一時式のlower完了後に一度だけ`rtlc`の型付きIRへ変換・保持し、以降のcapture剪定/ID remapも同じtyped IRを更新する。bodyがnullのpending定義を先にdeserializeせず、文字列opのdefault走査へ置き換えるだけにはしない。TSは全unionのswitchとnever検査に加え、wrapperを含む全fieldを必須keyにするtyped descriptorを使う。新variantだけでなく新しい子fieldも未分類なら型検査を失敗させる。source location等の非式fieldも明示して分類する。

上表の全v2解析・変換はこの入口のedgeを選んで処理し、自前のObject.values/Object.entriesや別の子リストを残さない。`limited/size`・`action_dimensions/limited`と`formulaAtom/formulaAlways`・`atom_expression/always_expression`も利用対象であり、lowering後の予算検査という理由で除外しない。resolver等の限定的な解析も、全edgeを列挙した上で許可roleを選び、他の形を明示拒否する。v1のlowering/走査は分岐を分けて凍結する。列挙はソース順、DAGは反復stackを保ち、宣言identity、capture順、target signature順を変えない。

| role / scope                                    | 各解析の動作                                                                                                                                                                                                                                                     |
| ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| value / condition                               | free-oldは読む。targetのactionとmembership用途は止める。IF/CASEのconditionは常に値位置                                                                                                                                                                           |
| result / action-body                            | And/Orのoperand、IF/CASEの結果枝、LET/Label bodyはactionを継承。ENABLED bodyはactionを開始。membershipはIF/CASE/LET/Labelの結果だけ継承                                                                                                                          |
| binder-domain / binder-body                     | domainは元の環境の値位置、bodyはbinderを導入した環境。action継承はExists.bodyだけ、Forall/CHOOSE/function/filter/map bodyは値位置。domain/bodyへmembershipを伝えない                                                                                             |
| generative-target / subscript                   | action内のUNCHANGED対象とsquare/angle subscriptだけ静的targetへ解決。通常式の同じ位置は値比較。SPECIFICATION/fairnessの変数要求も現行resolverを使う                                                                                                              |
| membership-operand / membership-result          | in/notinの右をmembership開始点とする。左は値位置。incoming用途はSeqの要素、FunctionSet.range、RecordSetのfieldへ継承し、domainや通常演算では止める。generativeな所属代入は従来どおりinだけ                                                                       |
| definition-body / call-actual / call-capture    | Function headはbounds+bodyへ正規化。free-oldは未使用LET定義も読む。要求解析は到達callのsummaryをformal/captureへ対応付け、actualをcaller環境で読む。未使用定義を新たにlowerしない                                                                                |
| EXCEPT-base / EXCEPT-key / EXCEPT-replacement   | baseとpathはcallerのold、replacementだけ自身のoldを束縛。free-oldはこの違いだけで判定し、calleeへcaller oldを移さない                                                                                                                                            |
| frame-actual / frame-capture / frame-body       | actual/captureはcaller、body/branches/formulaは新frame。capture/localScopeはbody境界を越えない。levelはbody summaryが要求するactual/captureだけを合成し、read/prime/enabledの固定levelを保つ                                                                     |
| action-branch / action-step / action-frame-body | 展開集計はbranch.stepsとaction frame.branchesだけを選ぶ。全stepを1個ずつ数え、frame wrapperも1 stepとし、さらにそのbodyの最大step数を足す。frame-actual/frame-capture、guard/assign/bind等の値式、その内部のENABLED・式frame・temporal frameは展開集計へ入れない |
| prime-argument / assignment-target / call-edge  | primeのphase検査、代入先の構文制限、callの実在ID/DAGを維持。DAG・remap・depth・構造検査は未使用actualと全frame bodyも読む                                                                                                                                        |

action展開の解析契約は、各branchの経路数をそのaction frame bodyの経路数の積（frameなしなら1）、branch集合の経路数をその和、最大step数を各branchの「直接step数＋各action frame bodyの最大step数」の最大値とする。actual/captureを数えないのはこの解析の位置規則であり、構造検証やfree-old等で読まないという意味ではない。4096経路/256 stepsを受理し、超過は従来の`action expansion exceeds 4096 branches or 256 steps`で拒否する。Rustの経路数4097とframe-body step加算時257への飽和も維持する。`B(a) == (x' = a) \/ (x' = a)`の`B(0)`を12個/13個連言するcompile-only対照（4096/8192経路）、直接256/257 steps、および`B(a) == x' = a`の1 frame＋254/255個の直接guard（wrapper込み256/257 steps）を保持する。actual/capture内のENABLED等を増やしても集計値が増えず、body内のframeを増やすと増える対照も置く。PROPERTY分類は列挙されたtemporal frameのformula edgeだけを選び、従来どおりframeを保ってatom/alwaysを判定する。

generative分類はactionを継承する枝だけを肯定材料にし、値を返すENABLED内部をgenerativeCallの肯定へ混ぜない。equality/Apalache assignment/inの構文target、next phaseのUNCHANGED/square/angle、Function定義をfalseとする規則も維持する。boxed-action判定とSPECIFICATION分解も列挙済みedgeから従来許可するalias/box/連言/fairnessだけを選び、一般式へ受理を広げない。

これで3e-dのhead/明示FunctionConstruction同値、caller binder/old、値と生成のUNCHANGED、用途伝播、未使用actual、frame境界を再現する。意味の訂正は既知5種（関数定義域、CHOOSE domain、notin RHS、CASE condition、uint32 count）と、C3で追加するTS明示v2の有限CHOOSE受理である。列挙される全子を全解析で無条件に評価する変更ではない。CASE条件の値評価、CHOOSE domain/predicateの評価と候補束縛、in/notinの同じ所属先は[固定TLCのTool.java（BoundedChooseはL1630–L1713）](https://github.com/tlaplus/tlaplus/blob/5a47802b5c391f59ecdd44117981f4ff8c0656ba/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/Tool.java#L1630-L1713)とも照合する。LET自由oldの拒否とmembership descriptorは本実装の契約であり、TLC内部方式とは区別する。

### role-tableと生成corpus

両frontendは列挙と同じdescriptorからplain data tableだけを公開し、node walkerや解析結果をtest用APIへ公開しない。表は`space / constructor / operator-class / position / role / scope / forwarding`を含み、葉の空行も持つ。Rustの小さなexampleで表をJSON化し、TS testがTSの表との完全一致をassertする。共通38種の意味edgeをcanonical化して完全一致させる。TSの41 kindとRustの38 kindの差（OperatorReference/Lambda/Subexpression）、LET INSTANCE/substitution、Fairness.enabledは別のparser-only行として両表へ明記する。Rustにないvariantを式edgeと偽らず、syntax/refusal境界はparserテストで、派生annotationは型/adapterテストで検査する。TSの未使用LETを一律の構文拒否へ変えず、通常parserが生成しないFairness.enabledをTLAの到達coverageへ数えない。tableの相互一致に加え、独立の期待role表と、実際に各位置へ届くprobeで両方同じ誤りになるのを防ぐ。

#### spelling inventoryを生成表から独立に閉じる

棚卸しの母集団はTSのインストール済み`tla-ts 0.0.10`の`src/lexer.ts`（BACKSLASH_OPERATORS / SYMBOL_ALIASES / UNICODE_OPERATORS / SYMBOL_OPERATORS）、`src/parser.ts`（INFIX_OPERATORS / POSTFIX_OPERATORS / DEFINABLE_PREFIX_OPERATORSと構文分岐）、`src/ast.ts`のExpression union、およびRustの`crates/rtla/src/lexer.rs`（ALIASES / SYMBOLSと`/=`・`=<`の別途正規化）、`parser.rs`、`ast.rs`である。loweringは`src/tla-safety.ts`のexpression・domain・membershipSet/membership・action・temporal・fold・target/SPECIFICATION経路と、`crates/rtla-compiler/src/lib.rs`の対応経路をすべて突き合わせる。TSはTypeScript AST APIでunionと定数表を読み出し、Rustはenum・match・文字列定数を機械抽出した。lowering文字列の抽出には診断・IR opも混じるため、その全件を受理一覧とはせず、分岐条件・import・arity・contextを確認して分類した。既存built frontendのtiny入力も補助対照とし、ビルド済み成果物だけから現在のsource対応範囲を決めない。実行コマンド、抽出script、raw出力、全spelling→canonical→constructor→frontend/contextの一覧は今回の外部報告に保存する。

機械抽出ではTSのinfix表91 entry、Rust44 entry（TSにだけある47 entry）、postfix表は両者とも`^+`, `^*`, `^#`である。primeの`'`は別構文分岐なのでpostfix表の3件へ混ぜない。TSの41 constructorはRustの38種に`OperatorReference`, `Lambda`, `Subexpression`を加えた集合で、Rustだけのconstructorはない。任意の宣言名・callは無限の識別子集合なので`user call` schemaで扱い、組込み名の有限表とは分ける。

| 共通parser constructorの全分類                                                                                                                                                                                                                                                                                      | 現在のlower受理 / 今回の決定                                                                                                                               |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Number / String / Identifier / Apply / And / Or / Prime / Unchanged / Enabled / If / Case / Let / Forall / Exists / SetEnumeration / SetFilter / SetMap / Tuple / FunctionConstruction / FunctionSet / Record / RecordSet / Except / ExceptAt / FunctionApplication / FieldAccess / BoxAction / AngleAction / Label | 両frontend。phase・binder・型・target等の既存制約を維持。Prime/Unchangedのnext、ExceptAtのreplacement、Seq/Nat/Intのmembership等は受理できる位置で検査する |
| Always / Eventually / WeakFairness / StrongFairness                                                                                                                                                                                                                                                                 | 両frontendのtemporal経路。値/action位置の拒否を意味edgeの到達へ数えない                                                                                    |
| Choose（有限domainあり）                                                                                                                                                                                                                                                                                            | 現在Rustのみ。C3どおりTS明示v2へ追加して受理matrixを一致させる。domainなしは両拒否のまま                                                                   |
| Decimal / UnboundedQuantifier / Bracket / Product                                                                                                                                                                                                                                                                   | 両parserのみ、両lower拒否。role descriptorには残すがsourceの意味到達枠から外し、parser/refusal・手書きIRの補完へ分離する                                   |

次表は片側の受理を省略しない名前付きの境界台帳である。「scope外を維持」は当該frontendの受理/具体的な拒否を別IDで保存し、同一受理やparity成功にしないという決定であり、無条件に通過させるignoreではない。

| spelling / constructor / context                                                            | TS                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               | Rust                                          | 決定                                                                                                                                                                                                                                               |
| ------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `\div` / `%` / `^`（各固定2、両引数）                                                       | parserのみ                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | 値・guard・temporal atomでlower受理           | 各operatorを非共有scope外として固定。共有binary表へ入れない                                                                                                                                                                                        |
| `:=`（固定2、target/value）                                                                 | parser・generative分類のみ、lower拒否                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | Apalache下の値eq・生成assignを受理            | 非共有scope外。旧共有表の2位置を削除し、代入両向きのRust固有対照を境界枠に置く                                                                                                                                                                     |
| Variant（固定2） / VariantTag（固定1） / VariantGetUnsafe（固定2）                          | 通常callのparseは可能、Variants import / builtin lowerは拒否                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Variants下でlower受理                         | 3つを各別IDの非共有scope外。Variantのpayload専用bound導入も共有edgeに偽装しない                                                                                                                                                                    |
| Choose（有限domainのset/body）                                                              | 現lower拒否                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | lower受理                                     | 唯一の受理統一追加はC3のTS明示v2対応。v1は別に凍結                                                                                                                                                                                                 |
| Forallの`\A`, `\forall`, `∀` / Existsの`\E`, `\exists`, `∃`でbodyがtemporal formula         | 値quantificationのみ、temporal bodyはlower拒否                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | 定数有限domainのtemporal quantificationも受理 | 2 constructorのtemporal contextを名前付きscope外。共有位置probeは値bodyをatomへ包み、temporal quantification受理を要求しない                                                                                                                       |
| `∘`→`\o` / `‥`→`..` / `≡`→`<=>` / `↝`→`~>`（各固定2）                                       | parserと該当lowerが受理                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | lexer拒否                                     | 各raw aliasを非共有scope外。ASCII/共有aliasの到達でRust受理と読み替えない                                                                                                                                                                          |
| `-.(argument)`→Apply(`-.`)（固定1）                                                         | nonfix形式をparse/lower受理                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | lexer/parser拒否                              | 非共有scope外。共通のunary source `-argument`→Apply(`-.`)と区別。`-. argument`は両parser拒否                                                                                                                                                       |
| `→`（FunctionSet） / `↦`（FunctionConstruction・Record） / `∷`（Label） / `≜`（definition） | 対応syntaxとlower受理                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | lexer拒否                                     | 各aliasを名前付きsyntax境界へ。共有の`->`, `\|->`, `::`, `==` recipeで意味edgeを検査                                                                                                                                                               |
| `⟩_`（AngleActionの閉じdelimiter）                                                          | parse、nextのlower受理                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | parse拒否                                     | TS専用syntax境界。共有の`>>_`を維持。開きだけUnicodeの`⟨A>>_v`は両受理で下表へ含める                                                                                                                                                               |
| symbolic binaryのnonfix呼出し（37形式）                                                     | `=>(left,right)`, `<=>(left,right)`, `~>(left,right)`, `=(left,right)`, `#(left,right)`, `<(left,right)`, `>(left,right)`, `<=(left,right)`, `>=(left,right)`, `\in(left,right)`, `\notin(left,right)`, `\subseteq(left,right)`, `\cup(left,right)`, `\cap(left,right)`, `\(left,right)`, `..(left,right)`, `+(left,right)`, `*(left,right)`, `\o(left,right)`, `\union(left,right)`, `\intersect(left,right)`, `\circ(left,right)`, `\equiv(left,right)`, `\leq(left,right)`, `\geq(left,right)`, `/=(left,right)`, `=<(left,right)`, `∈(left,right)`, `∩(left,right)`, `∪(left,right)`, `≥(left,right)`, `⇔(left,right)`, `⇒(left,right)`, `≤(left,right)`, `≠(left,right)`, `∉(left,right)`, `⊆(left,right)`をparse/lower受理 | symbolic nonfix呼出しをparse拒否              | 各名前付きsyntax-contextをscope外として固定。共有infixの位置数へ足さない。TS専用aliasの`∘(left,right)`, `‥(left,right)`, `≡(left,right)`, `↝(left,right)`も既存の4境界ID群へ含める。`-(left,right)`は両parser拒否で、unary minusの共有受理とは区別 |
| Label.params（`L(a):: body`）                                                               | parseしbodyをlower                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               | parameter付きlabelをparse拒否、paramsは空のみ | parameter付き形式は非共有scope外。共有Label.bodyは`L:: body`で到達させる                                                                                                                                                                           |
| OperatorReference / Lambda / Subexpression                                                  | parser variantあり、通常lower/fold参照は拒否                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | variantなし、対応構文をparse拒否              | 3つを各名前付きparser境界へ。foldは両者とも2-formal Identifier参照でbodyへ到達させる                                                                                                                                                               |
| LET INSTANCE / substitutions                                                                | TS ASTに存在、LET INSTANCEはlower拒否                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | LETには非対応、parse拒否                      | 各syntax/refusal境界を維持。未使用LETの一般拒否へ広げない                                                                                                                                                                                          |
| Fairness.enabled                                                                            | TS ASTの派生annotation、通常parserは生成しない                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | fieldなし                                     | annotation/adapter検査だけ。source reachへ数えない                                                                                                                                                                                                 |
| EXTENDS TLC / EXTENDS Variants                                                              | lowerのmodule検査で拒否                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | importを許可                                  | 2つを各名前付きimport境界。TLCのPrint/Assert/ToString等のbuiltin lower受理を意味しない                                                                                                                                                             |

共通parserのみの演算子は`-+->`, `@@`, `:>`, `<:`, `...`, `++`, `%%`, `|`, `||`, `--`, `**`, `&`, `&&`, `/`, `//`, `^^`, `\cdot`, `^+`, `^*`, `^#`で、両lowerの非対応拒否を維持する。Productの`\X`, `\times`, `×`、UnboundedQuantifierのdomainなし`\A` / `\forall` / `∀` / `\E` / `\exists` / `∃`および`\AA`, `\EE`、domainなしCHOOSE、Decimal、Bracketも各名前付きrefusalであり到達ではない。

TSだけのinfix表entryは以下の47個すべてをparser境界台帳へ固定し、各々Rustの構文拒否とTSのlower拒否を記録してscope外を維持する（alias正規化前の表entryも含む）: `!!`, `##`, `$`, `$$`, `(+)`, `(-)`, `(.)`, `(/)`, `(\X)`, `-|`, `::=`, `=|`, `??`, `\approx`, `\asymp`, `\bigcirc`, `\bullet`, `\cong`, `\doteq`, `\gg`, `\ll`, `\odot`, `\ominus`, `\oplus`, `\oslash`, `\otimes`, `\prec`, `\preceq`, `\propto`, `\sim`, `\simeq`, `\sqcap`, `\sqcup`, `\sqsubset`, `\sqsubseteq`, `\sqsupset`, `\sqsupseteq`, `\star`, `\subset`, `\succ`, `\succeq`, `\supset`, `\supseteq`, `\uplus`, `\wr`, `|-`, `|=`。

TSだけの残りのUnicode aliasは以下を各raw spelling名でscope外として記録する: `←`, `⁺`, `≈`, `≔`, `≍`, `◯`, `⩴`, `●`, `⋅`, `≅`, `÷`, `≐`, `…`, `‼`, `≫`, `⫤`, `≪`, `⊣`, `⊙`, `⊖`, `⊕`, `⊘`, `⊗`, `⇸`, `≺`, `⪯`, `∝`, `⁇`, `⊨`, `⊢`, `∼`, `≃`, `⊓`, `⊔`, `⊏`, `⊑`, `⊐`, `⊒`, `⋆`, `⊂`, `≻`, `⪰`, `⊃`, `⊇`, `⊎`, `‖`, `≀`。これらはlowerを新規実装せず、parser/token受理・具体的なlower拒否とRustのlexer/parser拒否を区別する（`←`はsubstitution構文側のtokenであり値operatorの受理ではない）。上の受理alias・構文aliasと合わせて辞書の片側差をすべて分類する。TRUE/FALSE/BOOLEANは共通値builtin、Nat/Intは共通domain/membership専用、Real/STRINGは未定義builtinとして両拒否。一般の宣言済み識別子はこの拒否対象へ含めない。

実装時も生成表とは独立にparser/lexer/ASTとlower分岐から再抽出したinventoryを固定source digest付きmanifestへ保存し、`shared reached / named nonshared / parser-or-context refusal / annotation`の排他的な分類で全entryを覆う。未分類・重複分類・新しいspelling/constructor/contextの未登録は生成前に失敗させる。共有受理のarity-position集合と下表の生成ID集合の双方向差分をassertし、期待role表と生成表の両方から同じspellingが消えても検出する。raw spelling、canonical AST operator、arity、context、source positionを別keyにし、lexer aliasをAST名で重複排除しない。

#### 共有受理の生成位置

共通sourceの38 constructor・52 position-schemaという構造inventoryは維持する。そのうちApply.args[_]の1 schemaをarity表へ置き換え、parser-onlyのUnboundedQuantifier.body / Bracket.args[_] / Product.factors[*]の3 schema（具体5位置）はrefusal補完へ移す。残る非Apply 48 schemaを70位置へ具体化し、共有raw aliasの26位置を足す。固定arityは全位置を独立schemaにし、同種可変長だけを2要素・両位置で代表させる。表中の可変配列の添字i/j/kは各0,1とし、nested配列は全組合せを作る。

| Apply以外のconstructor class                               | 位置                                                                                      | 具体位置数 |
| ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ---------: |
| Number / String / Identifier / ExceptAt                    | 子なし。4葉のsentinelは補完枠。Decimalはparser-only                                       |          0 |
| And / Or（`/\`, `\/`）                                     | 各operands[0], operands[1]                                                                |          4 |
| Prime / Unchanged / Enabled / Always / Eventually          | 各expression。primeはsource `'`、temporalは`[]`, `<>`                                     |          5 |
| If                                                         | condition, consequent, alternative                                                        |          3 |
| Case                                                       | arms[i].condition, arms[i].body, other（有無の対照）                                      |          5 |
| Let                                                        | definitions[0], definitions[1], body                                                      |          3 |
| Forall / Exists（`\A`, `\E`、有限の値body）                | 各bounds[0].set, bounds[1].set, body                                                      |          6 |
| Choose（C3で共有化する有限domain）                         | set（domain）, body（predicate）                                                          |          2 |
| SetEnumeration / Tuple（`<<...>>`）                        | 各elements[0], elements[1]                                                                |          4 |
| SetFilter                                                  | bound.set, predicate                                                                      |          2 |
| SetMap                                                     | expression, bounds[0].set, bounds[1].set                                                  |          3 |
| FunctionConstruction / FunctionSet                         | bounds[0].set, bounds[1].set, body / domain, range                                        |          5 |
| Record / RecordSet                                         | 各fields[0].value, fields[1].value                                                        |          4 |
| Except                                                     | function（1）, updates[i].path[j].Index.args[k]（8）, updates[i].value（2）               |         11 |
| FunctionApplication / FieldAccess                          | function, args[0], args[1] / record                                                       |          4 |
| BoxAction / AngleAction（`[A]_v`, `<<A>>_v`）              | 各action, subscript                                                                       |          4 |
| WeakFairness / StrongFairness / Label（WF_, SF_, `::`）    | 各subscript, action / body                                                                |          5 |
| **canonical非Apply小計**                                   | **旧75 − parser-onlyの5位置**                                                             |     **70** |
| And / Orの共有alias `\land`, `∧` / `\lor`, `∨`             | 4 spellingそれぞれoperands[0], operands[1]                                                |          8 |
| Forall / Existsの共有alias `\forall`, `∀` / `\exists`, `∃` | 4 spellingそれぞれbounds[0].set, bounds[1].set, body                                      |         12 |
| Always / Eventuallyの共有alias `□`, `◇`                    | 各expression                                                                              |          2 |
| Tupleの共有delimiter alias `⟨...⟩`                         | elements[0], elements[1]。両delimiterを同じrecipeで検査                                   |          2 |
| AngleActionの共有開きalias `⟨A>>_v`                        | action, subscript。閉じの`⟩_`は非共有                                                     |          2 |
| **Apply以外合計**                                          | **70＋共有alias 26。delimiterの混在全組合せはparser補完であり別の意味位置に水増ししない** |     **96** |

| Apply operator-class / raw spelling      | arityと独立位置                                                                                                                                                                                                                                | spelling展開後の位置数 |
| ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------: |
| user call                                | 宣言ごとにarityを検査。同種actual列は2-formal recipeのargs[0], args[1]、0/1/3-formalの対照は補完枠                                                                                                                                             |                      2 |
| in / notin（`\in`, `∈` / `\notin`, `∉`） | 4 spelling各固定2: args[0] left, args[1] membership RHS                                                                                                                                                                                        |                      8 |
| Seq                                      | 固定1: args[0] element domain。membership contextで受理させる                                                                                                                                                                                  |                      1 |
| equality `=`                             | 固定2: args[0], args[1]。両向きtargetを区別。非共有`:=`は含めない                                                                                                                                                                              |                      2 |
| ordinary unary（12 spelling）            | 各固定1: args[0]。`~`, `\lnot`, `\neg`, `¬`, unary `-`, `DOMAIN`, `Cardinality`, `Len`, `Head`, `Tail`, `SUBSET`, `UNION`                                                                                                                      |                     12 |
| ordinary binary（31 spelling）           | 各固定2: args[0], args[1]。`+`, binary `-`, `*`, `#`, `/=`, `≠`, `<`, `<=`, `\leq`, `=<`, `≤`, `>`, `>=`, `\geq`, `≥`, `=>`, `⇒`, `\cup`, `\union`, `∪`, `\cap`, `\intersect`, `∩`, `\`, `\subseteq`, `⊆`, `\o`, `\circ`, `<=>`, `\equiv`, `⇔` |                     62 |
| **leads-to `~>`**                        | **固定2: args[0] antecedent, args[1] consequent。両側の4probeとreached assertionをPROPERTYで要求する**                                                                                                                                         |                  **2** |
| range `..`                               | 固定2: args[0] lower, args[1] upper                                                                                                                                                                                                            |                      2 |
| Append                                   | 固定2: args[0] sequence, args[1] element                                                                                                                                                                                                       |                      2 |
| SubSeq                                   | 固定3: args[0] sequence, args[1] from, args[2] to → IR subseq.sequence/from/toを独立検査                                                                                                                                                       |                      3 |
| ApaFoldSet / ApaFoldSeqLeft              | 各固定3: args[0] operator参照, args[1] initial, args[2] collection。参照位置は解決した定義bodyへのadapterで4probeを到達させる                                                                                                                  |                      6 |
| **Apply合計**                            | **13 class（in/notin・2 foldは別class）。raw 54 spelling、unary/binaryの`-`を別arityとして55 entry、user-call代表を足して56 entry。特殊位置28＋unary 12＋binary 62**                                                                           |                **102** |

unary source `-argument`は両parserがApply(`-.`)へ変換する。raw `-.`を共通spellingとして数えない。既存Apply75位置からは、非共有`:=`の2・raw `-.`の1を除き、`~>`の2・`\neg`の1・`\circ`の2・`\equiv`の2・`=<`の2・共有Unicode Apply aliasの21を足して102になる。And/Or/Always/Eventually/Forall/ExistsをApplyへ重ねて数えない。CardinalityはFiniteSets、sequence/foldはSequences/Apalache等の既存import条件をrecipeへ固定する。

foldのargs[0]へ任意の値式を挿した構文拒否を、operator bodyへ到達したcoverageの代用にしない。参照先body、2 formal、callerのcapture/binderを使うadapterと、不正参照・不正arityの拒否対照を別IDで固定する。Function headはbounds+bodyの明示構築とのadapter対照、EXCEPTのField stepは非式annotation対照も持つ。

独立した小さなTLA+ recipeから、上表の各具体位置ごとに (1) LET定義内の自由な@、(2) 外側binderをactualにするcall、(3) parameter経由のUNCHANGED/ENABLEDへのstate要求、(4) parameter経由のmembership集合、を一つずつ挿す。set/数値/Boolean/actionの型に合わせたwrapper、使用/未使用、内外で異なるbinder/old、state actual/値actual、Nat/有限集合の正常対照を事前固定する。SubSeqは3位置それぞれへ4probeを入れ、特に`F(p) == SubSeq(<<1>>, 1, IF 0 \in p THEN 1 ELSE 0)`の`F(Nat)`と有限集合の対、及びLET内の自由@が第3引数にだけある拒否対照を持つ。固定arityの位置数と可変配列2要素の規則を混ぜない。葉もsentinel例を持つ。生成IRの各位置・anonymous expression/action/temporal frameにはTLA recipeと小さな手書きIRの補完表を持ち、どの位置がどちらで検査されたかassertする。構文上受理できない形は期待reason・拒否位置を固定し、受理した意味位置のcoverageと区別する。

`~>`は左右を交換した次の表を必須にする。共通の`Init == x = 0`、`Next == UNCHANGED x`、`CHECK_DEADLOCK FALSE`を使う。記法`T_i(P)`はi=0なら`P ~> TRUE`、i=1なら`TRUE ~> P`である。probe 2–4の2contextはPROPERTY式の中でhelperをcallする形と~>自体を通常actual付きtemporal helper frame内に置く形、probe 1は0-formal/1-formalの未使用LET定義とし、対照を含む各入力で解析が読む位置を固定する。

| ~>の各i=0,1へ入れるprobe   | recipeと対照                                                                                                                                                                        | 各側のreached assertion                                                                                                                                                                                                                                                      |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1: LET自由old              | `Safe == [<<1>> EXCEPT ![1] = LET unused == T_i(@ = 1) IN TRUE][1]`をINVARIANT Safeへ。1-formal版は`unused(a)`。対照は唯一の`@`を`1`へ置換                                          | parseした`Apply("~>").args[i]`内に唯一の自由`@`があり、両frontendの`protocol / Unsupported TLA input: LET definitions with free EXCEPT @`を要求。反対側はTRUE、他の拒否原因なし。対照はcomplete。未使用でもfree-oldが訪問した証拠とし、temporalの値lower受理の証拠にはしない |
| 2: caller binder actual    | `B(p) == p = 9`と`T_i((\A n \in {9}: B(n)) = TRUE)`をPROPERTYへ。対照のactualは`B(9)`。第2contextは`F(p) == T_i((\A n \in {p}: B(n)) = TRUE)`、`Live == F(9)`で同じactual対照を作る | 両args[i]の値atom内のquantifier→call.actualを独立期待IRで照合。caller n=9とcalleeのlocal/bound slotを分離しcomplete。quantifierの外側を`= TRUE`でatom化し、Rustのみのtemporal quantificationへ入らない                                                                       |
| 3: parameter経由state要求  | `U(p) == ENABLED (UNCHANGED p)`と`T_i(U(x))`をPROPERTYへ。対照は`T_i(ENABLED (UNCHANGED x))`。第2contextは`F(p) == T_i(U(p))`、`Live == F(x)`とし、対照はU(p)を直接ENABLEDへ置換    | args[i]からcalleeのENABLED/UNCHANGED targetへ到達し、callerのstate slotへ解決されたbranchとframe/actualを独立期待IRで照合。4 checker経路のcomplete。値actual `U(0)`のtarget拒否は別boundary IDで、reachの代用にしない                                                        |
| 4: parameter経由membership | `F(p) == T_i(x \in p)`、`Live == F(Nat)`をPROPERTY Liveへ。対照は`F({0})`。もう一方のcontextは`M(p) == x \in p`、`Live == T_i(M(Nat))`で有限集合を対照にする                        | args[i]のmembership RHS用途がformal pからactual Natへ届く期待descriptor/IRを照合。Nat/有限集合とも3 compile受理、両IR×native/Wasmの4 checkでcomplete・states1/generated2/depth0・reason/violationなし                                                                        |

各`(~>, i, probe, context, control)`の期待IDについて、source ASTの位置・sentinelの一意性、該当解析の期待結果または唯一の自由oldの具体的拒否、受理時の独立期待IRとchecker結果をassertしてからreached ledgerへ記録する。ID集合を生成前manifestと完全一致させ、左右の片側だけ、IR always/eventually/orだけ、または`~>`を値/actionへ置いた未対応拒否だけではreachedを立てない。自由oldのrefusal reachedと受理したsemantic reachedは別fieldで集計する。これはtest用walker結果APIの追加を要求せず、plain role table・source AST・外部から観測するcompile/check結果で検査する。

今回の既存built frontendの最初のcontext対照では、左右のmembership Natは4経路ともcomplete・states1/generated2/depth0/work39、有限集合はwork41、自由oldは3 compileで上記拒否、置換対照は4経路complete・work12だった。binderを値atomで包んだrecipeは4経路work40、state要求は左work42/右work41、直接ENABLED対照は両側work38で一致した。上表のもう一方のcontextも左右・4probe・対照を実行し、受理28入力×4経路の計112 check応答は各入力内で完全一致、自由oldの4入力×3 compileは指定拒否だった。これらは正常版の小入力確認であり、未実装のcorpus/reached assertionの成功やTLC由来oracleではない。先に試した裸のtemporal quantifierはTS/Rustでworkが異なったため、そのcontext差も外部報告へ保存し、共有recipeには値atom版を採用した。

TS compile、Rust native/Wasm compileの受理または具体的なtyped refusalを比較し、受理時は正規化IRを独立templateの期待構造と比較する。ID、capture、branch/step順を消す正規化は行わず、従来のoptional空配列とnull subscriptだけを正規化する。生成されたTS/Rust両IRをnative/Wasmでcheckし、各経路で診断を含む完全応答一致と独立した意味結果を要求する。共通protocol拒否だけで成功にせず、期待reasonと到達位置を検査する。期待値はrole/TLA意味論から固定し、frontendの出力やproduction tableから生成しない。

件数枠は上表から`(96 + 102)位置 × 4probe × 2受理context/解析context × 2対照 = 3168 source入力`と再導出する。通常の値/生成contextが受理できないoperatorには、~>のPROPERTY直接/helperや自由oldのLET解析など、上記の指定contextを使う。値/actionの未対応拒否でcontext枠を埋めない。既に具体化した配列位置へさらに2を掛けず、ternaryも3位置のまま数える。補完1696は葉・parser/非共有境界/head/option/参照adapter256、IR/frame768、wire境界384、既知再現・CHOOSE追加対照・action上限288を維持し、合計上限を4864入力へ改訂する。parser-onlyへ移した5位置も最初の補完枠で検査し、共有reachの分子へ戻さない。全期待ID・実数と位置対応を実装時の実行前manifestへ固定し、生成欠落・重複・枠超過は失敗にする。未使用予約枠を実行件数に数えず、必要な位置を上限へ合わせて削らない。各sourceは2 KiB・16定義・2binder・recipe深さ4以内を維持し、action上限のcompile-only入力も簡潔な連言でこの範囲へ収める。checkはmaxDepth8/maxStates100/maxWork10000。

ビルド/TLC採取を除く全corpusのhard deadlineは180秒、1入力の全経路処理は1秒を維持する。共有sourceの3168入力だけでも3168/180 = 17.6入力/秒、最大22176呼出しが必要で、補完込み上限4864なら4864/180 = 約27.02入力/秒となる。受理sourceは3 compile＋両IR×native/Wasmの4 checkで最大7呼出しなので、全入力がその形なら34048呼出し、約189.16呼出し/秒が必要になる（補完は各経路数もmanifestへ固定し、この最悪上限を超えない）。これは予算からの必要速度であり、完走の実測保証ではない。table/生成/worker準備の費用も180秒へ含める。TS/Wasmはworker、nativeはchild processを外側watchdogから中断できるようにし、timeout時は当該worker/processを再起動する。当該IDのtimeoutと未実行IDを区別して失敗保存する。通常はnativeの既存JSONL processを再利用し、各workerでWasmを1回ロードする。seedや件数を結果後に減らさず、例外・timeout・boundedをcompleteへ変換しない。180秒で全期待IDが終わらなければ未完了であり、samplingへ切り替えない。

明示v2ではTSも有限CHOOSEを値として受理し、既存Rustのlower・有限domain・predicate・遅延評価と同じfragmentに揃える。既存のchoose IR/decoderを使い、新wire variantは追加しない。domainはcaller環境、predicateは候補binderを導入した環境で読む。CHOOSEのincoming action/membership用途はdomain/predicateへ伝えず、その内部にあるin/notin等の開始点は通常どおり解析する。次の対照を受理matrixへ含め、TS/Rustの同一受理と意味結果一致を完了条件にする。

| 有限CHOOSEの対照                                                   | 先に固定する期待                                                                                                                                                                           |
| ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| domainとpredicate各位置の4probe、scalar/tuple/nested               | sourceとIR binding.domain/bodyの到達を別々に要求。自由oldのLET拒否と有限集合の正常対照を分ける                                                                                             |
| caller binderをdomain actualにする`D(p) == CHOOSE n \in p: TRUE`   | `\A n \in {9}: D({n}) = n`。calleeの同名nとcallerのnを混同しない                                                                                                                           |
| caller binderをpredicate actualにする`P(p) == CHOOSE n \in {4}: p` | `\A n \in {9}: P(n = 9) = 4`。actualはcaller環境、候補はcallee環境                                                                                                                         |
| CHOOSE自体をactualに渡すdomain/old対照                             | `I(p) == [<<4>> EXCEPT ![1] = p][1]`に対し`[<<9>> EXCEPT ![1] = I(CHOOSE n \in {@}: TRUE)][1] = 9`。caller old=9をcallee old=4へ取り違えない                                               |
| CHOOSE自体をactualに渡すpredicate/old対照                          | 同じIで`[<<9>> EXCEPT ![1] = I(CHOOSE n \in {9}: n = @)][1] = 9`。未使用actual版は評価せず、使用版と対にする                                                                               |
| 非空domainで無解 / 空domain                                        | compileは受理し、要求時は両IR×native/Wasmで`protocol / CHOOSE: no element of the domain satisfied the predicate`。complete・empty verdictやcompile拒否へ置き換えない。未選択枝は評価しない |

省略/明示v1のTS `expression Choose`拒否、Rust v1の既存受理挙動、全既存v1 serialize bytesは凍結し、明示v2だけにTSのlower分岐を追加する。無限/無制限CHOOSEは受理へ広げない。この受理matrixのCHOOSE mismatchを既知差の保存だけで完了にしない。その他の非共有構文・contextは上の名前付き境界台帳どおりに扱い、有限CHOOSE統一から全fragmentのparityを主張しない。未使用化してprobeが届かない形をcoverageへ加えず、frontend別の現状記録を意味oracleやparity成功にしない。

binder/old/target/membershipの既存TLC goldenを再利用する。CHOOSEは`evals/runs/0046-finite-verified/`の`choose-order`、`choose-equivalent`、`choose-tuple`、`choose-nested`、`choose-capture`、`choose-no-witness`、`choose-empty`、`choose-lazy`、`choose-predicate-error`、`choose-tuple-error`の各`model/Differential.tla`・`Differential.cfg`・`tlc.log`と`reports.json`/`manifest.json`をdigest付きで参照し、保存runを編集しない。無解/空domainは保存TLCのmodel-error（exit75）、未選択枝はsuccessを使う。選択順の互換性を新たに広げず、新規tiny例はsingleton domainにする。

新規TLCはCASE値条件・notin formalの2本に加え、上表のdomain caller binder、predicate caller binder、domain caller old/actual、predicate caller old/actualの4本と、それぞれ比較値だけを0・0・4・4へ変えた初期Safe違反の4対照、計10本を固定revision・既存capture手順で採取し、expected-step3fへ追加する。共通Initはx=0、NextはUNCHANGED x。4正常版のsuccessと4違反版の初期反例は採取前の予測であり、今回は未採取。TLCが出さない計数を補わない。free-oldの拒否政策とwire上限にTLC successを捏造せず、work/IR構造もTLC由来とはしない。

### wire境界

v2 programの全count/indexは共通の`WIRE_INDEX_MAX = 4294967295`（0以上の整数）を用いる。対象はDefinitionのid/parameterCount/captureCount/localScope、3種frameのparameterCount/captureCount/localScope、call.definition、parameter/capture/bound/readのslot、Bindingとbind stepのslot/tupleSize、assign/choose stepのslot、fold.accumulator。配列index/長さをwireの数へ変換するときも同じ契約を適用する。TS decoderとRustのwire deserialize/typed IR validationへ同じfield表を適用し、usizeへの変換前に検査する。Rustをnative/Wasmで別上限にしない。表のuint32上限もparity対象にし、既存のbounds/resultのuint32契約は維持する。

全fieldのmax/max+1をTS decoderとnative/Wasmへのraw requestで検査し、Rustで直接作ったIRにも同じ検証を行う（max+1をusizeに保持できる64-bit nativeで直接検査し、Wasmはraw requestで検査）。未使用DefinitionのparameterCount/captureCountはmax受理、max+1拒否を小入力で確認する。他の巨大ID/slotやframe countはrange境界のunit testと全programの意味検査を分け、maxでも実在ID/arity/localScope等に反すれば既存のtyped refusalを要求する。localScope40/41、arity・slotの有効/無効対照も持つ。巨大配列を作ってmaxの正当性を装わない。

### scope・Red・実装・検査順

今回の編集はこのnoteだけ。以下は次stepの実装対象であり、今回は変更しない。

- 列挙器と呼出し側: `src/tla-safety-analysis.ts`（新規）、`src/tla-safety.ts`（有限CHOOSEの明示v2 lowerも含む）、`crates/rtla-compiler/src/analysis.rs`（新規）、`crates/rtla-compiler/src/lib.rs`（action集計・PROPERTY分類も移行）。
- wire: `src/safety-program.ts`、`crates/rtlc/src/ir.rs`、`crates/rtlc/src/safety_definitions.rs`。
- tests: `evals/tlc-differential/analysis-parity-corpus.ts`と`analysis-parity-test.ts`（新規）、`evals/tlc-differential/frame-decoder-test.ts`、`evals/tlc-differential/operator-sharing-test.ts`、`evals/tlc-differential/regression-test.ts`、`crates/rtla-compiler/tests/analysis_parity.rs`（新規）、`crates/rtla-compiler/examples/analysis_roles.rs`（新規）、`crates/rtlc/src/safety_definition_tests.rs`。
- 独立期待表: `evals/fixtures/tlc-operator-sharing/analysis-roles.json`（新規）。tiny capturesは同directoryの`sharing-case-condition-step3f`、`sharing-notin-formal-step3f`、`sharing-choose-domain-caller-step3f`、`sharing-choose-predicate-caller-step3f`、`sharing-choose-old-domain-argument-step3f`、`sharing-choose-old-predicate-argument-step3f`、及び後4件の名前へ`-violation`を付けた計10 case directoryの各`Differential.tla`・`Differential.cfg`、各case名に`.log`を付けた同directory直下の10ログと`expected-step3f.json`（全て新規）、同`README.md`。0046の再利用元は読取のみ。契約説明は`docs/safety-ir-v2.md`と`docs/rust-frontend.md`。
- support同期の対象は`crates/rtla-compiler/SUPPORT.md`、`crates/rtlc/SUPPORT.md`と`.jev-lint/checklists`の両crateの`.manifest.json` / `.record.json`。保存record/digestを手編集しない。

先に独立role期待表、全具体位置の4probeと到達assertion、wire境界と既知5種、有限CHOOSEの受理/意味結果をテストへ追加する。意図したRedと既存Greenを次に固定する。

| 先行テスト                                                                             | 現treeへの意図したRed / 保持するGreen                                                                                                                                         |
| -------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| production descriptor由来tableと独立期待表・全位置IDの照合                             | 総列挙/tableが未実装。C1の集計・PROPERTY利用対象、C2のSubSeq.to、C4の~>両側と共有aliasを欠く表/生成器も失敗させる。parser/lower機械inventoryからの未分類・両方向の集合差もRed |
| Rust CHOOSE.setの自由old、notin formal/capture/head、TS CASE condition                 | 既知の拒否漏れ・用途漏れ・過剰target要求でRed                                                                                                                                 |
| 全count/indexのwire境界                                                                | nativeの未使用DefinitionのparameterCount/captureCountのmax+1受理でRed。他の有効/無効対照も維持                                                                                |
| TS明示v2の有限CHOOSE（domain/predicate、binder/old、actual、無解/空domain、lazy）      | 現TSの`expression Choose` compile拒否でRed。無解/空domainはcompile受理後の評価エラーを要求するため、同じprotocol codeだけでは合格させない                                     |
| SubSeqのargs[0]/[1]/[2]各4probe、特にtoのmembership/free-old                           | 現frontendの挙動はGreen。第3位置の未生成をcoverage Red、将来そのedgeを落とせば意味assertionのRedとして検出する                                                                |
| ~>左右の4probe、PROPERTY membershipとLET自由oldの各対照・reached集合                   | 現frontendの小入力はGreen。spelling・左右・probe・context・controlの欠落をcoverage Redにし、値/action拒否の代用もRedにする                                                    |
| action 4096/8192経路・256/257 steps・actual/capture除外、temporal helperのPROPERTY分類 | 現挙動はGreen。移行時にframe-bodyの集計を落とす、actual/captureを加える、formula edgeを落とす変更を検出する                                                                   |
| 関数定義域の使用/未使用/nested/ENABLED、head/explicitのtarget/membership、v1 identity  | 3e-dで既にGreenの対照とv1 bytes/拒否を保持する。新規CHOOSE例も省略/明示v1の現挙動を別assertionにする                                                                          |

実装順は(1)独立期待表・上記Redと既存Greenの固定、(2)typed列挙・arity/adapter・production role table、(3)free-old、target/membership/generative、TS明示v2の有限CHOOSE lower、(4)IRのlevel/capture/call/phase/depthとaction展開集計・PROPERTY分類、(5)wire検証、(6)固定TLC対照と全corpus、の順とし、各段で既存対照と新coverageを確認する。CHOOSE受理追加に既存Rustの受理範囲縮小を使わない。

最後に3 crateのfmt/clippy/locked all-targets tests、`just compiler-test`、生成corpus、operator-sharing cohort、frame decoder、v1 identity、`just safety-test`、`just portable-build` / `just portable-test`（実Chromiumを含む）、`just quint-test`、`just check`、`just browser`、`just evaluate` / `just replay-evaluation`を実行する。just/pnpmは隔離userconfig付きとし、既存mutation対象または共有テストを変えたら`just mutation`も行う。`just support-plan` / `just support-sync` / `just support-check`も行うが、キーなしskipと既知stale manifest失敗は成功へ読み替えない。新しいrunは追記し、stage・履歴/refs・GitHubへの書込みは行わない。今回の設計改訂ではこれらを実行せず、本文差分・件数計算・編集範囲と不変条件を読取検査する。

v1のbyte identity、既存goldenと乖離記録の唯一性/digest、frontend40・compiled/decoder128・checker式48・値64・動的128、binder40・branch4096/step256とwork課金/予算は不変。このcorpusが保証するのは列挙したprobeと対照の範囲であり、全programのparityや全TLA+互換の証明ではない。

## step 3f-b1: systematic analysis parity の先行テスト

本番解析の変更前に、独立role表373行、source inventory再抽出と654分類、共有198位置の4probe・2context・2対照を固定した。sourceの3168 IDと、補完877 ID（parser/context 209、IR/frame 343、wire 194、既知再現・CHOOSE・action等131）を実行前manifestへ保存した。合計4045 IDは4864枠内で、未使用819枠を実行数やcoverageへ加えない。補完のfrontend別assertionは同じsourceからの独立モデルとは数えない。source/config digest、補完source digest、登録名と全位置を双方向照合する。許可した本番変更はTS/Rustの空role表stubとRustのmodule/exportだけで、列挙器・解析・CHOOSE・wireの実装は次stepに残した。

source ASTの指定位置・唯一のsentinel、caller binderとcallee binder、Natと有限集合のactual、独立したIRのfield投影と定義参照、~>の左右のalways/or/否定/eventually構造を検査する。semantic reached、自由old拒否、未使用定義の正常対照を別fieldで保存し、全IDを台帳へ残す。SubSeqの3位置、共有alias、~>両位置を省略しない。sourceのTS/Wasmをworkerへ、補完を隔離processへ置き、1入力1秒の外側watchdog、全corpus180秒、再起動と未実行の区別を保つ。native JSONLとWasm instanceはsource corpusで再利用する。Rustの新規65 unit testも固定名で個別に1秒の外側watchdogから実行する。2 KiB、16定義、frameごとのbinder slot 2、probeへのrecipe深さ4、check予算8/100/10000と既存上限は変更しない。

先行実行のNode全corpusは4172 tests中4064 Green、106 assertion Redと集約2件のRed、timeout・未実行・skip・TODOなしで約5.3秒だった。source3168 IDは全件実行し、3142通過・26 Red。CHOOSE未対応24件とRust CHOOSE.setの自由old拒否漏れ2件である。これは全fragmentのparity達成ではない。補完999 assertionは921通過・78 Red。Rustは65 tests中31 Green・34 assertion Redだった。Redの名前と失敗assertionは指定の外部reportと台帳に全件保存する。

| Redのassertion名の系列                                                                    | 件数 | 意図した失敗                                                                  |
| ----------------------------------------------------------------------------------------- | ---: | ----------------------------------------------------------------------------- |
| production TS/Rust role table、analysis_parity_role_table_matches_independent_expectation |    3 | 独立373行に対してstubが空配列                                                 |
| generated Chooseのbinder/target/membership、controls CHOOSE/reused/tinyのTS               |   78 | compile受理を要求し、現状はexpression Choose拒否                              |
| generated Choose.set old、controls CHOOSE domain free old、Rust choose_domain_free_old    |   14 | LET definitions with free EXCEPT @拒否を要求し、現状は受理                    |
| controls notin formal/capture/function head/construction/tiny、Rust notin                 |   14 | Natのmembership用途を要求し、現状はoperator Nat拒否                           |
| controls CASE condition/tinyのTS                                                          |    2 | 条件の値位置を要求し、現状はgenerative UNCHANGED requires state variables拒否 |
| wire native未使用Definition count max+1、Rust range/direct IR                             |   29 | uint32超過拒否を要求し、現状はusize deserialize 25件とcount経路で受理         |

Nodeの106件とRustの34件は140 assertionであり、欠陥140種という意味ではない。既知の関数定義域対照、SubSeq.to、~>左右、actionの4096/8192経路・256/257 steps、actual/capture除外、PROPERTY分類と既存v1はGreenのまま保持した。新規CHOOSE20例の省略/明示v1は3frontendの120 digest/refusal検査を追加し、既存baselineは変えない。

固定TLCを1本ずつ256 MiB・60秒・1 workerで10本採取した。CASE/notinと4 CHOOSE正常版はsuccess・各1状態/生成2。比較だけを0/0/4/4へ変えた4本は、予測した初期反例ではなくexit151のconstant invariant拒否だった。ログにないtraceや計数を補わず、expected-step3fへinput-errorとして保存した。0046の10 CHOOSE対照とreports/manifestの計32ファイルはdigest付きの読取参照である。既存golden/log/v1/乖離記録は不変。

既存検査はsafety-test 1183、operator-sharing 970、frame decoder 10、v1 identity subset 257、browser 3が通過した。compiler-testは新規7 Redで停止するが、既存69 testsを新規名の除外で検査し、benchmark smokeも通過した。all-targetsへ一律に--skipを渡した最初の補助commandはcriterionの引数拒否になったため、そのログを保持してlib/tests/examplesとbenchを分けた。just checkは既存の設計noteのformatで初回失敗し、内容を維持した整形後に154 testsが通過した。evaluate/replay-evaluation、support-planも実行した。support-syncはAPI key未設定の警告skip、support-checkは既存stale manifestによる失敗を保持し、同期済みとは扱わない。mutationは今回の明示禁止に従い実行しない。stage/unstage、履歴・refs、GitHubへの書込みは行わず、全変更を未コミットで残す。

## step 3f-b1b: CHOOSE の状態レベル違反対照

既存の4本の`*-step3f-violation`はconstant invariantのinput-error（exit151）のまま保存し、正常版のSafeの比較先だけを状態変数xへ変えた4本を追加した。名前は`sharing-choose-domain-caller-step3f-state-violation`、`sharing-choose-predicate-caller-step3f-state-violation`、`sharing-choose-old-domain-argument-step3f-state-violation`、`sharing-choose-old-predicate-argument-step3f-state-violation`である。singleton domain、Init、Next、configは元の正常版と同一。指定captureで1本ずつheap256m・timeout60・workers1を用い、全4本がexit12の初期Safe違反とx=0を出力した。状態数・生成数は印字されず、expectedには補っていない。

追加12テストはnative/Wasmの8件がGreen、TSの4件が既存のexpression Choose拒否でRedだった。operator-sharingは970/970、checkは共通挙動のVitest77＋node:test77、実Chromiumは3/3。既存source3168件の台帳は完全一致し、Rustの31 Green・34 Redも不変だった。固定manifestの更新案を使う一時検証用コピーではNode全体4184件中4072 Green・112 Red（独立110＋集約2）、補完1011件中929 Green・82 Redで、既存Redの名前・失敗内容に変化はなく、新規RedはTSの4件だけだった。

固定manifestはsource digestと登録一覧を完全一致で検査する。既存expected entriesの変更禁止との整合を確認中のため、manifest原本は変更していない。原本の全体実行は補完source digest不一致で停止し、補完の個別実行でもmanifest一覧不一致が1件追加される。上記の全体件数は更新案を使った検証結果であり、原本の成功とは扱わない。更新案は既存ケースを保持した4 ID・12登録の追記と派生digest・件数だけである。本番・既存capture/log/expected case・v1 baseline・乖離記録は不変、全変更は未コミットで、stage/unstage・履歴/refs・GitHubへの書込みとmutationは行っていない。

## step 3f-b2: 型付き列挙器の途中成果と凍結inventoryによる停止

phase (2)でTS/Rustにsource・typed IR・wrapperのrole付き子列挙器を追加した。production role表373行は独立期待表と両方完全一致し、TS role / Rust role / Rust role unitの3件がGreenになった。本体の解析呼出し側にはまだ接続していない。Rustにはposition schemaと実列挙の二重管理が残り、phase (2)の設計要件全体も完了とはしない。phase (3)〜(5)の挙動・IR移行・wire検証には進んでいない。

凍結`analysis-inventory.ts`の384〜394行は、必須編集対象の`src/tla-safety.ts`と`crates/rtla-compiler/src/lib.rs`を含むsourceSha256、およびlowerEvidenceを含む抽出全体を凍結`analysis-inventory.json`へ完全一致させる。このため本体実装と全凍結ファイル不変・既存Green維持が両立しない。外部コピーで、未変更本体はverifier成功、TS本体末尾へASCII空白1文字だけを加えると`analysis inventory source digest changed`で失敗することを再現した。実treeの本体・テスト・manifestを変更せず、「凍結テストに問題があれば停止して報告する」という今回の指示に従って停止した。時間切れではない。

停止時のNode corpusは4184件中4074 Green /110 Red（独立108＋集約2）。source3168件は3142 Green /26 Red、補完1011件は929 Green /82 Red。Rust新規65件は32 Green /33 Red。残る独立Redはsource26＋補完82＋Rust33の141件で、開始時Greenからの回帰は0。Node＋Rustの全corpusは6.392秒、180秒以内、timeout・未実行・skip・cancel・TODOは0だった。operator-sharingは970/970、9.478秒。

3 crateのfmt/clippyは最終成功し、locked all-targetsはcompilerが既知parity6 Red、rtlcが既知wire27 Red、idealis-coreは18/18だった。compiler既存69件とcompatibility bench、frame既存10件、v1 identity257件、safety1183件、quint13件、checkのVitest77＋node:test77、通常Chromium3件、portable Chromium2件は成功した。frame全体は151 Green /既知wire2 Red、portable-testはrtlcの既知wire27 Redで停止し、全体成功とは扱わない。evaluateは新規外部directoryへ出力して保存witnessを再生した。support-planは7 crates /28比較、support-syncはkeyなし警告skip、support-checkは既知stale manifestのexit3である。

凍結62ファイルは開始前・終了時とも全てOK、HEAD・refs・indexと両frontend本体hashも不変。契約の挙動はまだ変更していないため契約docsは更新せず、support記録/digestも手編集していない。全変更は未コミットであり、stage/unstage・履歴/ref・GitHub書込・mutation・新TLC採取は行っていない。全Red名・assertion・再現証拠・実行ログは今回指定の外部reportへ保存した。

## step 3f-b2c: source解析の単一列挙と明示v2 CHOOSE

phase (2)を完了した。Rustの`descriptor_specs()`による二重の位置一覧を撤去し、型付きAST/IRの代表値を同じ`analysis_children`へ渡してproduction role表を導出する。aliasとparser-only metadataは位置の実列挙と区別して保持する。TS/Rustのproduction表373行は独立期待表へ完全一致し、既存source走査を列挙器へ接続した。

最終レビューでedge.depthのmetadataも旧walkのparent expression/frame加算規則へ揃えた。TSで受理した2776生成programと独立frame fixtureの全56056式位置について、従来walkとのpath別深さ一致を補助検査した。392件の既定拒否を受理・意味成功へ数えない。これはmetadataの検査であり、既存IR解析を移行するphase (4)の完了ではない。

phase (3)ではfree-old、target要求、membership要求、generative分類と限定target resolverをrole・scope・forwardingに基づく列挙へ移した。LETの未使用定義・関数headのdomainもfree-oldで読み、EXCEPT replacementだけがoldを束縛する。CASE条件のaction要求を止め、Rustのnotinにもmembership用途を伝える。TSの明示v2へ有限CHOOSEを追加し、caller domainとbinder導入後のpredicateを区別する。caller binder/old、actual、tuple、空domain・無解の評価時エラー、lazy・未使用actualを固定対照で検査した。省略・明示v1のTS拒否は維持する。

| 検査                | 前回b2               | phase (2) | phase (3) | 最終再抽出後 |
| ------------------- | -------------------- | --------- | --------- | ------------ |
| Node parity         | 4074 Green / 110 Red | 4085 / 99 | 4180 / 4  | 4181 / 3     |
| Rust新規65 tests    | 32 Green / 33 Red    | 34 / 31   | 38 / 27   | 38 / 27      |
| source 3168 ID      | 3142 Green / 26 Red  | 3144 / 24 | 3168 / 0  | 3168 / 0     |
| 補完1011 assertions | 929 Green / 82 Red   | 939 / 72  | 1009 / 2  | 1009 / 2     |
| operator-sharing    | 970 / 970            | 970 / 970 | 970 / 970 | 970 / 970    |

phase (2)/(3)のNode件数は旧inventory source hashによる1失敗を含む。最終はNodeの独立Red2件とその補完集約失敗1件、Rust27件である。残る独立Red29件は、native raw requestのDefinition.parameterCount/captureCount超過2件と、Rustの全25 wire fieldのrange unit・直接IR count超過2件。前回の独立Red141件から112件がGreenへ移り、source・補完・Rustの開始時Greenからの回帰は0。全既存IDを対応付け、timeout・未実行・skip・cancel・TODOを合格へ変換していない。

最終のNode corpusは7.023秒、Rust個別実行を含むcorpus全体は7.444秒で180秒以内。source実測20608呼出し、source台帳6.929秒、補完台帳3.061秒で、並行区間を加算しない。operator-sharingは9.252秒。固定TLC再利用30 assertionsとtiny 42 assertionsも全てGreenで、新しいTLC採取は行っていない。

`analysis-inventory.json`は凍結extractorの`extractAnalysisInventory()`だけで`extracted`を再生成した。ほかのtop-level 10 fieldsは不変で、専用guardはexit0、凍結verifierの再抽出と分類の双方向差分もGreen。分類・共有位置・context表は編集していない。ほかの凍結61ファイルは開始時・終了時とも全てOK。

phase (4)/(5)は未着手である。RustのContext/Entry・一時式・lowering戻り値を含むtyped IR保持への移行と全IR解析の接続には、JSON構築160箇所・約50関数を跨ぐ変更とv1互換の検査が必要で、残る時間内に完了・検証する見込みが立たなかった。解析ごとの再deserializeで完了に見せず、compile可能なphase (3)境界で止めた。Rustのtyped更新adapterと、列挙器のedge.depthを既存深さ解析へ適用する作業もphase (4)へ残る。phase (6)の固定TLC・全corpus検査は実行したが、wire Redが残るため完了・parity達成とは扱わない。

最終境界検査では3 crateのfmt/clippy、compiler locked all-targets・`just compiler-test`、core18 tests、`just safety-test`1183、v1 byte identity257、`just portable-build`、Chromium portable2 / browser3、Quint13、`just check`のformat/typecheckと共通挙動77×2、評価・保存witness再生が成功した。rtlc all-targetsは77 Green /既知wire27 Redでexit101、`just portable-test`も同じ理由で停止し、後続の実Chromium2件は別途検査した。frame decoderは151 Green /同じnative count2 Red。評価はrecipeと同じpnpm scriptへ新規外部directoryを明示して実行した。初回`just check`のimport順序エラーは修正し、元ログを保持した。

`just support-plan`は7 crates /28 comparisons、`just support-sync`はAPI key未設定の警告skip、`just support-check`は既知stale manifestのexit3。support record/digestを手編集していない。履歴・refs・index・GitHubを変更せず、全変更を未コミットで残した。mutation・依存追加・limit変更・code comment追加・新TLC採取は行っていない。

## step 3f-b2d: wire上限とtyped IR解析の完了

phase (5)を先に完了した。TS/Rustに`WIRE_INDEX_MAX = 4294967295`を置き、wireの全25 count/indexへ同じ範囲を適用する。Rustはu64として読んで上限を検査した後にusizeへ変換し、native/Wasmの上限を分けない。直接構築したIRでもdefinition count、frame signature、slot・binding・fold等へ同じ検証を行う。実在ID・arity・localScope等の既存拒否は維持する。未使用DefinitionのparameterCount/captureCountはmaxを受理し、max+1を拒否する。残っていた独立Red29件は全てGreenへ移り、Node4184/4184、Rust対象65/65、source3168/3168、補完1011/1011、operator-sharing970/970、frame decoder153/153となった。この段階のNode＋Rust corpusは6.707秒で180秒以内だった。

phase (4)はGreenの境界を確認しながら進めた。TSのlevel、capture/localScope、call収集・DAG・ID remap、prime/temporal phase、compiled depth、限定read resolver、action展開集計、PROPERTY分類を型付き子列挙と既存のimmutable更新adapterへ接続した。Rustは読み取りと更新のIR列挙を同じexhaustive matchのマクロから導出し、完了した定義と最終programのcapture剪定・call remap・DAG・level・prime・depthを先に移した。その後、action/temporal/program、lexical環境、式・domain・membership・foldの一時値まで型付きIRへ移した。pending定義のbodyはOptionで保持し、nullを先にdeserializeしない。v2の解析ごとのJSON反射走査・再deserializeは撤去し、同じtyped treeを更新する。v1のJSON loweringと走査は既存経路へ分けて維持した。

actionの集計はbranch.stepsとaction frame.branchesだけを列挙から選ぶ。frame wrapperの1 stepとbodyの最大step数、経路数の積・和、4097/257への飽和を維持し、actual/captureと値式内部のENABLEDは展開集計から除く。PROPERTYはtemporal frame.formulaのedgeを選び、frameを再構成してatom/alwaysを分類する。Rustの一時式移行でSequenceInの右辺にSequenceSetを余分に包む回帰を構造比較が検出した。source12件・補完1件と集約2件、operator-sharing5件の失敗ログを保持し、従来どおりSeqの要素descriptorを渡す形へ修正した。凍結テストは変更していない。

TS移行、Rustの最終typed変換、action/PROPERTY/program、lexical環境の各境界でNode4184/4184・Rust65/65・operator-sharing970/970を確認した。最終の全corpusも同じ件数でGreen、source3168/3168・補完1011/1011、開始時Greenからの回帰0、timeout・未実行・skip・cancel・TODOは0だった。最終Node＋Rust corpusは広域検査と並行して9.139秒、Node単体8.784秒、operator-sharing10.110秒で、180秒のcorpus上限を維持した。source実測20608呼出し、source台帳8.679秒、補完台帳4.496秒で、並行区間を加算しない。固定TLCの再利用30 assertionsとtiny42 assertionsもGreenで、新しいTLC採取は行っていない。phase (6)の固定対照と全corpusまで検査した。これは列挙済みprobeと対照の範囲の証拠であり、全programのparity・全TLA+互換・未知バグ発見の証明ではない。

最終検査は3 crateのfmt/clippy/locked all-targets、compiler-test、v1 byte identityとbaseline coverage257/257、frame decoder153/153、safety-test1183/1183、portable-build、portable-test（実Chromium2/2を含む）、Quint13/13、checkのformat/typecheckとVitest77＋node:test77、browserの実Chromium3/3が成功した。評価と保存witnessの再生はrecipeと同じscriptへ新しい外部出力先を渡して成功した。compiled depthは受理2776生成programと独立frame fixtureの全56056式位置を従来walkとpath別に比較し、392件の既定拒否を成功へ数えていない。

inventoryのextractedは凍結extractorだけで再生成し、他のtop-level10 fields・classifications・共有位置・context表を維持した。専用guardはexit0、凍結verifierのfresh extractionと分類の双方向差分もGreen。他の凍結61ファイルは開始時・終了時とも61/61 OK。support-planは7 crates/28 comparisons、support-syncはキー未設定による警告skip、support-checkは既知stale manifestのexit3であり、同期・鮮度検査成功へ読み替えない。support record/digestは手編集していない。履歴・refs・index entries・GitHubを変更せず、全変更を未コミットで残した。mutation・依存変更・limit引上げ・code comment追加・新TLC採取は行っていない。v2の対象IR解析の独自walkは残しておらず、legacy v1のwalk、source loweringの構文分岐、checker自身の検証/実行walkは今回のfrontend IR解析移行と区別する。

## step 3f-b3: call-actual要求の先行回帰とinventory再凍結

第五回レビューのP1-1に対し、本番を変更せず、共通の手書きfixture14入力を補完テストとRust parityへ追加した。membershipの未使用formalへNatを渡す例は3frontendとも`operator Nat`拒否を要求し、有限alias版は通常値の同一定義を共有する4定義の配列を固定期待とする。`Unused(p) == p`と`Probe({0})`は受理対照とした。受理例は省略可能な空配列とnull subscriptだけを正規化し、TS/native/Wasmの全v2 IRを照合する。両frontendのIRはnative/Wasmの4 checker経路で完全応答とcomplete・states1/generated2/depth0を要求する。

同型sweepはmembership、ENABLED内target、nextのgenerative use、LET内free-oldについてformal使用/未使用を含む。targetはcalleeのUNCHANGEDがformalを要求する場合だけstate actualへ解決する。generativeは未使用actual内のUNCHANGEDを肯定材料にせずguard callを保ち、callee自身のUNCHANGEDではaction frameとstate代入を要求する。両者の値target拒否も固定した。free-oldはsummary需要の有無によらず全actualを構造走査する契約に従い、自由@を両方で拒否し、actual内のEXCEPTで束縛した@は両方で受理する。free-oldを需要駆動へ広げた設計とは解釈しない。未決定の設計を推測で補ったケースはない。

初回のgenerative-unused-actualの手書き期待を`0 = 0`としてしまい、通常値IRの`0 = prime(0)`を落としていた。既存のprime保持契約へこの1 nodeだけを訂正し、入力・受理・guard/no-frame期待・予算・件数は変えていない。初回ログと訂正条件を保存し、この4 assertion（Node3/Rust1）を製品のRedに数えない。

最終結果はNode parity4236件中4230成功、5個別Redと補完親1失敗、Rust79件中77成功・2 Red。個別Redは`membership-unused-nat: native/Wasm`の期待拒否に対するcompiled受理、`membership-unused-finite: native/Wasm`の固定4定義に対する5定義、同finiteのnormalized v2 IR不一致である。Rustでは`analysis_parity_call_actual_membership_unused_nat`と`analysis_parity_call_actual_membership_unused_finite`が同じ拒否・定義assertionで失敗した。これは2種の欠陥の複数経路であり、7つの独立欠陥とは数えない。source3168入力、既存補完1011 assertion、既存Rust65件はGreenを保った。新規補完52 assertion中47成功・5 Redで、補完全体は1058/1063。全sourceは5.303秒、20608呼出し、補完は2.575秒、timeout・未実行・skip・cancel・TODOは0。operator-sharingは970/970。sourceのsemanticReached2384、refusalReached392、controlReached392を区別する。

U1はextractorのRust lowering入力へ`typed.rs`、`typed_lower.rs`、`typed_expr.rs`をlib.rsの隣へ追加し、同じ入力群をlowerEvidenceへ連結する最小変更とした。抽出アルゴリズムや分類表は変えず、inventoryのextractedだけ再生成した。source digestは8から11ファイルへ、RustのlowerEvidenceは文字列7・関数41追加、constructor追加0。verifierは成功し、317 spelling・TS41/Rust38 constructorは全て既存分類で覆われ、欠落0、extracted以外のtop-level fieldsは不変だった。

manifestは新規登録名・route上限とRust名を実行前に固定した。source3168＋supplement919＝4087 logical ID、最大24437呼出しで既存上限内。旧61凍結ファイルに新しい共通fixture1件を加えた62件をSHA-256で再固定し、inventoryは引き続き一覧から除外して別copyへ保存した。変更した既存凍結ファイルはRust parity、補完controls、extractor、controls/corpus manifestの5件だけである。

cargo fmt/clippy、`just check`、`just browser`の実Chromium3件、外部出力先を指定したevaluate/replay-evaluationのrecipe相当は成功した。support-planは7 crates/28比較、support-syncはAPI key不在の警告skip、support-checkは既知stale manifestでexit3を保持する。これらを成功へ読み替えず、support成果物のstageは行わない。本番ソース、分類、既存golden・TLCログ・v1 baseline、Git履歴・refs・indexを変更せず、全差分は未コミットのままとした。新しいTLC採取、mutation、依存変更、予算引上げは行っていない。

## step 3f-b4: call-actualのmembership過剰伝播と文書の修正

第五回レビューのP1-1を修正した。Rustの`membership_demands`でuser-callのactualを事前に走査するときだけincoming purposeをfalseにし、callee summaryのComposeが要求したformal・captureを合成する経路を維持した。全actualの構造走査、actual内部の所属式によるmembership開始規則、他の解析は変えていない。TSのmembership、両frontendのtarget・generative・free-old、IRのlevel・capture・限定read resolver・phase/depthも点検した。targetは既にactualへのincoming actionを止め、generativeはcallee側を分類し、free-oldは未使用actualを含めて構造検査する。既存・b3テストで追加の差はなく、これらのconsumerは変更しなかった。

修正前にb3のNode5個別Redと補完親1失敗、Rust2 Redを再現した。修正後は7個別assertionと集約が全てGreenとなり、Node parity4236/4236、source3168/3168、補完1063/1063、Rust対象79/79、operator-sharing970/970となった。b3の全source ID・補完名・Rust名を対応付け、開始時Greenからの回帰は0。Node＋Rust corpusは5.700秒で180秒以内、source台帳は5.088秒・20608呼出し、補完台帳は2.400秒。semanticReached2384、refusalReached392、controlReached392を区別し、timeout・未実行・skip・cancel・TODOを成功へ置き換えていない。

`docs/rust-frontend.md`のfold説明は、v1のbody展開とv2の専用frame内2 bound slotをactualとする共有callへ区別した。compiler READMEのJSON構築説明を省略・明示v1へ限定し、明示v2は型付きIRを保持・解析・更新すると明記した。inventoryは凍結extractorでextractedだけ再生成し、変更はRust本体のsource digest1件だけである。他のtop-level10 fields・分類・位置表は不変で、専用guardはexit0、凍結verifierもGreen。凍結62ファイルは開始時・終了時とも62/62 OKである。

3 crateのfmt/clippy/locked all-targetsと`just compiler-test`、frame decoder153/153、v1 identity257/257、safety-test1183/1183、portable-build、portable-testの実Chromium2件、Quint13/13、checkのformat/typecheck・Vitest77＋node:test77、browserの実Chromium3件は成功した。evaluate/replay-evaluationはrecipeと同じpnpm scriptへ新規外部出力先を指定し、実行と保存witness再生が成功した。support-planは7 crates/28比較、support-syncはAPI key不在の警告skip、support-checkは既知stale manifestでexit3のままであり、同期や鮮度検査の成功とは扱わない。support記録/digestを手編集せず、stage/unstage・Git履歴/refs・GitHub書込みは行わず、全変更を未コミットで残した。新TLC採取・mutation・依存変更・limit引上げ・code comment追加はない。

## step 3f-b5: 深い公開IRの通常Dropに対する先行回帰

第六回レビューのP0-1に対し、本番を変更せず先行回帰を追加した。公開型付きIRはどの型にもcustom Dropがなく、標準の再帰Dropで解放される。再帰所有辺は次のとおり。`ir.rs`のExpression（Box、Vec、`Option<Box>`、`Vec<(String, Self)>`。Update・Binding・Domain・Branch・Stepを経由してExpressionへ戻る循環を含む）、`Step::Frame.branches`から`Branch.steps`へ戻る鎖、`TemporalFormula`の`Box<Self>`。さらに`Expression::Data`が所有する`Value`（`value.rs`のList・Record・Set・Map）も同じ標準Dropで再帰する。`TemporalFormula::Frame`のarguments/captures、Atom、boundsはExpression側への出口であり、`TemporalFormula`自身の再帰辺ではない。

`crates/rtlc/tests/deep_drop.rs`に54件を追加した。内訳はExpression直下33件、ENABLEDのsubscriptとstep経由（Guard・Assign・Bind・Choose・Frame）7件、invariant述語の根1件、Step::Frame.branches（ENABLED・initial・next）3件、TemporalFormula 5件、Value 5件で、同じ形の兄弟variantは代表1件で覆う。各テストは深さ100000の鎖を公開APIだけで一つ構築し、8MiBの明示stackを持つスレッドの中で公開`check`に渡し、期待する拒否をassertしてから通常のdropまでを行い、スレッドがpanicなしにjoinすることを要求する。拒否は式・action枠が`Safety expression nesting exceeds 48`、時間式の鎖が`Temporal formula nesting exceeds 40`、入れ子の時間量化が`Invalid temporal binding count`である。Valueの鎖は評価されないDataに置くため`check`が拒否せず、`check`が返ることとdropだけを要求する。`crates/idealis-core/tests/deep_request_drop.rs`は同じaction枠の鎖を`Engine::handle`の`CheckSafety`要求として渡し、ライブラリ内部のdropを経たうえで`protocol`拒否のMessageを要求する。既存の`public_program_deep_action_frames_are_checked_before_recursive_analysis`は`std::mem::forget`を外し、assertionを変えずに通常dropまで検査する。

修正前の結果は、新規55件と既存1件の計56件がすべてstack overflowでabortする意図したRedである。panic表示はなく、拒否のassertionがdropより前に成立している。Valueを含め修正前から通る族はない。既存の1件は既定の2MiBテストスレッドで5000段のdropがdebugビルドで溢れる。各テストは単独プロセス・60秒の外側timeoutで実行し、abortが他の結果を止めない。深さだけを1000へ変えた一時コピーでは55件すべてがGreenになり、拒否の期待とdropがdepth以外で満たされることを確認した。一時コピーは削除した。

Greenを保ったのは、Node analysis parity 4236/4236、Rust parity全manifest 79/79、operator-sharing 970/970、frame decoder 153/153、v1 byte identityとbaseline 257/257、`just safety-test`、`just compiler-test`、rtlcとidealis-coreのfmtとclippy `-D warnings`、56件を除くrtlc・idealis-coreの全テストである。56件を含むrtlcのlib・integration binaryとidealis-coreの`deep_request_drop`binaryは修正までprocessごとabortするため、`pnpm rtlc:test`経由の`just portable-test`も停止する。

凍結リストはb3の62件のうち変更した`safety_definition_tests.rs`のhashを更新し、新規2ファイルを加えた64件で、`analysis-inventory.json`は引き続き除外する。本番ソース、manifest、fixture、既存TLCログ、v1 baselineは変更せず、全差分は未コミットのまま、stage/unstage・履歴/refs・GitHub書込み・新TLC採取・mutationは行っていない。

## step 3f-b6 design: 公開IRと`Value`の有界stack破棄

改訂: 2026-11-17 初版（前提レビュー前の草案）。
改訂: 2026-11-17 第2版。前提レビューのNO-GO条件C1（`Value`を保護しないとData内の代入、`Scalar::try_from`、idealis-coreのValue所有型で再帰Dropが残る）に対し、`Value`にも同じworklistのDropを置く案(a)へ変えた。壊れる箇所をstubのDropを置いたcompiler検査で列挙し直し、Valueのby-value分解4ファイルの移行と、ownerが承認した`tests/host.rs:31`のpattern形の変更を加えた。所有辺の再点検表、単独破棄・field置換・Value経路・2つの深さとstack・割当計数を検査するテスト、README注記の4型化を加え、b5の内訳の誤記（deep_drop 55件→rtlc 54件・core 1件・既存lib 1件）を直した。切り離しの置き場所は4型を扱うため新しいprivate module `crates/rtlc/src/bounded_drop.rs`へ移した。

第六回レビューP0-1、b5の56 Red、前提レビューC1に対する設計。変えるのは破棄と、それに伴う5ファイル6関数のby-value分解の移行だけで、公開型のfieldとvariant、`check`、serde、等価性・hash・cloneの挙動は変えない。

### 再帰辺とDropを置ける型

自己再帰辺を持つのは3型だけである。`Expression`の`Box<Self>`・`Vec<Self>`・`Option<Box<Self>>`・`Vec<(String, Self)>`、`TemporalFormula`の`Box<Self>`、`Value`の`Vec<Value>`・`Vec<(String, Value)>`・`Vec<(Value, Value)>`。間接の循環は`Expression → Vec<Branch> → Branch.steps → Step → Expression / Domain / Vec<Branch>`の一つで、`Step::Frame.branches → Branch → Step`を含む。`Domain`・`Binding`・`Update`は`Expression`への出口で、`Expression`から`TemporalFormula`へ、`Value`から他の型へ戻る辺はない。自己再帰辺の循環上にはstdの`Box`/`Vec`と自分しかないため、単独で破棄される深い値を有界stackにするには、その型自身のDropが要る。`Value`も同じで、`Expression::Data`の中でだけ切り離しても、単独の`Value`、`Data.value`への代入、`Value`内部のVecのclearは親のDropを通らない。`Step`↔`Branch`の循環にはどちらか一方のDropで足りる。

### 比較

4型（`Expression`・`Branch`・`TemporalFormula`・`Value`）に空のDropを置いた作業木外のコピーで、rtlcに依存する全manifest（rtlc、rtla-compiler、idealis-core、`evals/shared-oracle-rust`）に`cargo check --all-targets --keep-going`を当て、壊れる箇所をcompilerで列挙した。lib・bin・test・example・benchを含み、cfg(wasm32)側はidealis-coreのwasm32 lib checkで覆った。doctestはない（コードブロックを持つdoc commentと`doc = include_str!`がない）。下表の移行を当てた後は、4 manifestの全targetのcheckとclippy `-D warnings`、wasm32 lib checkが通る。前版のscan.pyは型解析ではないため補助に留める。

| Dropを置く型                                                     | 壊れる箇所（E0509）                                                                                                                                                                                                                                         |
| ---------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Expression`・`Branch`                                           | 0。全て`&`/`&mut`経由か、`typed_expr.rs:96`の`I::Read { slot, next: false }`のようにCopyのfieldだけを束縛する                                                                                                                                               |
| `TemporalFormula`                                                | 2。`crates/rtla-compiler/src/typed_lower.rs`の`negate`（29行の`T::Atom`）と`combine`（38行の`T::Atom`の組）                                                                                                                                                 |
| `Value`                                                          | 4ファイル5箇所。`crates/rtlc/src/value.rs`の`normalize`（105行のmatch、7 arm）、`crates/idealis-core/examples/counter.rs`の25行と65行、`crates/idealis-core/tests/host.rs:31`、`evals/shared-oracle-rust/src/main.rs`の`native_value`（45行のmatch、3 arm） |
| `Step`・`Domain`・`Binding`・`Update`・`SafetyProgram`等の所有根 | Dropを置かない。前版の検索では`Binding`に1箇所（`typed_lower.rs`173–178行）                                                                                                                                                                                 |
| idealis-coreの`Request`                                          | Dropを置かない。`Engine::execute`の`match command.request`（lib.rs:469）が複数armでprogram・source等をmoveする                                                                                                                                              |

(a) 再帰型すべて（`Expression`・`Branch`・`TemporalFormula`・`Value`）にworklist型のDropを置く案は全経路を覆う。費用はValueのby-value分解4ファイル5箇所の移行で、うち`tests/host.rs:31`は既存テストのpattern形の変更になる。ownerはこの1箇所に限り、assertionを変えない互換編集を承認した。

(b) 所有根（`SafetyProgram`、`Definition`、`Predicate`、`TemporalProperty`）だけにDropを置く案は、壊れる箇所が0で56件もGreenになる。各テストは`SafetyProgram`全体を破棄するためである。しかし根へ入れる前や取り出した後の単独の値、根のfieldの置換・消去（`definitions[0].body = …`、`program.next.clear()`、`definitions.take()`等）は根のDropを通らない。Greenが実際より広い保証を示すため採らない。前版の3型案も同じ理由で、単独の`Value`、`Data.value`への代入、`Scalar::try_from`の拒否後、idealis-coreのValue所有型を覆えず、前提レビューC1で退けられた。

(c) その他。`stacker`等のstack拡張は再帰型へのDropを同じく要し、依存を増やす。wasm32のportable coreでは使えず、メモリもdepthに比例する。arena/index化、`Rc`化、子を包むnewtypeは公開型の形を変える。forget、globalな廃棄list、別threadでの破棄は、leak、global state、上限なしのいずれかになる。linked list式のtail loopは単一子の鎖しか扱えない。構築時の深さ制限では公開enumリテラルによる構築を止められない。確立した手法はregex-syntax 0.8.11の`impl Drop for Ast`（src/ast/mod.rs:1634）と`Hir`（src/hir/mod.rs:1913）である。子を空の葉と`mem::replace`してheapのworklistへ移し、子を持たないnodeは早く戻る。(a)はこの形である。

### 採用: `Expression`・`Branch`・`TemporalFormula`・`Value`にDrop、worklistは1本

Dropを持たせるのは`rtlc::ir::Expression`、`rtlc::ir::Branch`、`rtlc::ir::TemporalFormula`、`rtlc::Value`（idealis-coreが再exportする`idealis_core::Value`と同じ型）の4型。`Step`↔`Branch`の循環には`Branch`を選び、`Step`のby-value destructuringを残す。`Step`・`Domain`・`Binding`・`Update`・`Definition`・`Predicate`・`TemporalProperty`・`SafetyProgram`・`SafetyTraceStep`・`SafetyViolation`・`SafetyResult`、idealis-coreの型にはDropを置かない。これらの深い部分は必ず4型のどれかのDropの中で処理されるので、破棄は有界になる（下の再点検表）。4つのDrop、切り離し関数、worklistの型は新しいprivate module `crates/rtlc/src/bounded_drop.rs`の1か所に置く。

移行は次の6関数で、どれも出力・エラー・評価順を変えない。

| 箇所                                                   | 移行                                                                                                                                                                                          | 挙動が同じ理由                                                                                                        |
| ------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `crates/rtlc/src/value.rs`の`normalize`                | `normalize(mut self)`と`match &mut self`にし、各armでString・Vecのfieldを`std::mem::take`で取り出す。`value => Ok(value)`は`Self::Null \| Self::Bool(_) \| Self::String(_) => Ok(self)`にする | 取り出した値に同じ検査を同じ順で当てる。残る`self`は空のfieldだけを持ち破棄される。armの網羅は10 variantで変わらない  |
| `crates/rtla-compiler/src/typed_lower.rs`の`negate`    | `T::Atom`を`&mut formula`で照合し、`std::mem::replace(expression, I::Old)`で式を取り出す                                                                                                      | 出力の`Not`の形は同じで、`Old`を持つ旧Atomは破棄されるだけ                                                            |
| 同`combine`                                            | `match (&mut left, &mut right)`でAtomの組を照合して2式を`mem::replace`で取り出し、他の組は`_`のarmで元の`left`・`right`を包む                                                                 | 出力の`And`/`Or`の形と左右の順は同じ                                                                                  |
| `crates/idealis-core/examples/counter.rs`の`execute`   | 25行を`let Value::List(values) = &entry.state else`、65行を`match &action`（36行と同じ形）にする                                                                                              | 読取りだけで、`name == "add"`は`&String`と`&str`の比較として同じ結果。exampleの`#[test]`と`main`のassertionは変えない |
| `crates/idealis-core/tests/host.rs:31`                 | `let Value::Int(index) = &choice else`（1文字の追加、owner承認）                                                                                                                              | `index.parse()`は参照からも同じ`str::parse`。assertionは変えない。b5の凍結64件には含まれない                          |
| `evals/shared-oracle-rust/src/main.rs`の`native_value` | `native_value(mut value: Value)`と`match &mut value`にし、Stringは`mem::take`、Intは参照から`parse`、Recordのfieldsは`mem::take`して従来どおり再帰する                                        | signatureと出力JSONは同じ                                                                                             |

これらは移行前の型でもcompileするので、`host.rs`はテスト段で、他は実装段で変える。normalizeは`idealis-core/tests/contract.rs`の共有fixtureと`tests/rtlc.rs`、negate/combineはv1 byte identityとoperator-sharing、counterはexampleの`#[test]`、native_valueは`just cross-test`の既存検査が覆うので、移行のための新しいテストは足さない。

公開APIの費用。下流のcrateは4型から非Copyのfieldを値でmoveできなくなる。enum variantのpattern、`let Branch { steps, .. } = branch`のような構造体pattern、by-value束縛を持つguard付きarmを含む。参照での照合、Copyのfieldだけの束縛（`Expression::Read { slot, next }`、`Value::Bool(flag)`）、値の構築・clone・serde・値全体のmoveは影響を受けない。これは破壊的なsource互換性の変更なので、`crates/rtlc/README.md`の`rtlc::ir`の段落（34行）に次の旨を1文で加える。「`Expression`・`Branch`・`TemporalFormula`・`Value`は深さによらず有界なstackで破棄するため`Drop`を実装し、これらの値から非Copyのfieldをpatternでmoveできない。参照で照合し、`Vec`・`String`のfieldは`std::mem::take`、`Default`を持たない`Expression`・`TemporalFormula`・`Value`のfieldは`Expression::Old`・`Value::Null`などの葉との`std::mem::replace`で取り出す。」`Value`はidealis-coreの公開型でもあるため、`crates/idealis-core/README.md`の再exportの文（11行）にも、`Value`がDropを実装し参照・`std::mem::take`・`std::mem::replace`で扱う旨を英語で1文加える。どちらも`just support-plan`で確認する。

### 切り離しと上限

`bounded_drop.rs`にprivateの`enum Detached { Expression(Expression), Branch(Branch), Formula(TemporalFormula), Value(Value) }`と、各型を`&mut`で受けて葉でない子だけを`Vec<Detached>`へ移す`detach_*`関数を置く。4つのDropは、直接の子がすべて葉なら何もせず戻る。そうでなければ`Vec::new()`へ自分の葉でない子を移し、空になるまでpopして同じ関数を当てる。popしたnodeはそのiterationの終わりに破棄される。

| 辺                                                                      | 切り離し                                                                             |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `Box<Expression>`（argument・left/right・body等）                       | `mem::replace(&mut **child, Expression::Old)`で中身だけ移す。Boxは親と共に解放       |
| `Vec<Expression>`、`Vec<(String, Expression)>`                          | `mem::take`して要素（後者は`.1`）を移し、名前はその場で破棄                          |
| `Option<Box<Expression>>`（ENABLEDのsubscript）                         | `take()`して`*box`を移す                                                             |
| `Vec<Update>`、`Vec<Binding>`                                           | `mem::take`し、DropのないUpdate・Binding・Domainを値で分解して式を移す               |
| 埋込みの`Binding`（CHOOSE）・`Domain`（`In`）                           | `&mut`でDomainのBoxを`Old`と置換し、`Set`のVecを`take`する                           |
| `Vec<Branch>`（ENABLED、`Step::Frame`）                                 | `mem::take`してBranchを移す。Branchは`steps`を`take`し、DropのないStepを値で分解する |
| `Expression::Data.value`                                                | 葉でない`Value`を`mem::replace(value, Value::Null)`で同じworklistへ移す              |
| `Vec<Value>`（`List`・`Set`）                                           | `mem::take`し、葉でない要素を移す                                                    |
| `Vec<(String, Value)>`（`Record`）                                      | `mem::take`し、名前はその場で破棄して葉でない値を移す                                |
| `Vec<(Value, Value)>`（`Map`）                                          | `mem::take`し、keyとvalueの両方に同じ規則を当てる                                    |
| `Box<TemporalFormula>`                                                  | `mem::replace(&mut **child, TemporalFormula::Atom { expression: Expression::Old })`  |
| `TemporalFormula`の`arguments`・`captures`・`bounds`・`Atom.expression` | Expressionと同じ規則。`Atom.expression`は`Old`と置換する                             |

葉は、`Expression`では`Parameter`・`Capture`・`Bound`・`Old`・`UnmatchedCase`・`Literal`・`Read`と葉の`Value`を持つ`Data`、`Branch`では`steps`が空のもの、`TemporalFormula`では葉の式を持つ`Atom`、`Value`では`Null`・`Bool`・`Int`・`String`・`Bytes`・`ModelValue`とVecが空の`List`・`Set`・`Record`・`Map`である。直接の子は、Box・Vec・Optionで所有する4型の値と、DropのないStep・Update・Binding・Domainを経由して所有する4型の値である。判定はvariantとVecの空だけを見るので定数時間。葉はその場に残し、Vecから取り出した葉はすぐ破棄する。不変条件は「`detach_*`の後にnode内へ残る子はすべて葉」である。このためpopしたnodeの破棄で走るDropは直接の子がすべて葉の分岐で戻り、drop glueは葉だけを破棄する。stackの深さはDrop、loop、`detach_*`、葉の破棄の定数段で、入力の深さによらない。

各nodeはworklistへ高々1回入って1回処理され、処理は直接の子の数に比例する。時間はnode数に線形、worklistの最大長はnode数以下。確保はworklist自身の償却拡張だけで、node毎の確保はない。`mem::take`と`Old`・`Null`・`Atom { Old }`の葉は確保しない。直接の子がすべて葉なら何も確保しない。各nodeの葉でない子が1つの鎖ではworklistの長さが定数に留まり、確保回数は深さによらない。`Value`は破棄の頻度が高いので費用を分けて書く。scalarと、要素がすべて葉の容器（scalarのrecord・list等）は確保なしで戻る。要素に葉でない容器を持つ値は、外側の1回の破棄につきworklistを1回確保する。callのたびに`Argument::new`がcloneするactualも同じ規則に従う。破棄中の確保の失敗はRustの既定どおりabortする。

Dropは破棄時にだけ走る。field・variant・serde属性・derive・methodは変えない。`check`は`&self`で、Clone・Debug・PartialEq・Eq・Hash・Serializeのderiveは参照を読み、Deserializeは値を組み立てるだけである。したがって検査結果、wire bytes、等価性、hashは変わらない。置換した葉は破棄中のnodeにしか現れない。

### 所有辺の再点検

前提レビューの表の各辺と、idealis-coreでValueへ直接至る型を、採用した規則で確かめた。`lib.rs`・`finite.rs`・`history.rs`・`host.rs`・`shrink.rs`は`crates/idealis-core/src/`、`ir.rs`・`value.rs`は`crates/rtlc/src/`を指す。

| 所有辺・入口                                                                           | 位置                                                                                                                                    | 破棄を担うDrop                                                                                  |
| -------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| ExpressionのBox・Vec・record tuple・frame/callのactual・capture                        | `ir.rs:32`・`:46`・`:60`・`:74`・`:86`・`:118`・`:126`・`:189`・`:207`                                                                  | `Expression`。単独の値、field代入、Vecのclearでも入る                                           |
| Enabledのbranchesとsubscript                                                           | `ir.rs:194`                                                                                                                             | `Expression`が`Vec<Branch>`と`Option<Box>`を切り離す                                            |
| Step::Frame → Branch.steps → Step::Frame                                               | `ir.rs:309`・`:321`・`:366`・`:368`                                                                                                     | `Branch`。単独のStepやbranchesのclearも固定段数でここへ入る                                     |
| StepのGuard・Assign・Bind・Choose                                                      | `ir.rs:323`・`:331`・`:334`・`:339`                                                                                                     | `Branch`のdetachが値で分解し`Expression`へ                                                      |
| Domain・Binding.domain・Update.path/value                                              | `ir.rs:293`・`:354`・`:360`                                                                                                             | 固定段数で`Expression`                                                                          |
| Predicate・Definition・SafetyProgramの全IR field                                       | `ir.rs:375`・`:390`・`:406`・`:412`・`:416`・`:424`                                                                                     | 固定段数で`Expression`・`Branch`・`TemporalFormula`。代入・take・clearも同じ                    |
| SafetyProgramWire・DefinitionTable                                                     | `ir.rs:431`・`:442`                                                                                                                     | 同上。成功前とエラー経路も同じ                                                                  |
| TemporalFormulaのFrame・Atom・単項・二項・量化、TemporalProperty                       | `ir.rs:694`・`:708`・`:711`・`:720`・`:728`・`:741`                                                                                     | `TemporalFormula`、式は`Expression`                                                             |
| `Expression::Data.value`からList・Record・Set・Map                                     | `ir.rs:63`、`value.rs:17`・`:18`・`:19`・`:20`                                                                                          | 親のDropからは同じworklistで`Value`へ。単独のValue、Dataへの代入、内部Vecのclearは`Value`（C1） |
| SafetyTraceStep.values・SafetyViolation.trace・SafetyResult.violation                  | `ir.rs:603`・`:610`・`:630`                                                                                                             | `Value`                                                                                         |
| `Scalar::try_from`が拒否した後の引数                                                   | `ir.rs:643`                                                                                                                             | `Value`                                                                                         |
| CheckSafety・CheckSafetyParallel・`Message::Compiled`                                  | `lib.rs:155`・`:150`・`:220`                                                                                                            | SafetyProgramを経て上の3型                                                                      |
| Canonicalize・Start・StartLoaded                                                       | `lib.rs:194`・`:198`・`:167`                                                                                                            | `Value`                                                                                         |
| Replay・CheckLoaded・CheckFinite・Resume                                               | `lib.rs:201`・`:171`・`:186`・`:205`、Trace `:61`・`:62`、Step `:52`・`:53`、Response `:126`・`:127`、Transition `:40`・`:41`           | `Value`                                                                                         |
| LoadModel・StartFinite・CheckFinite                                                    | `lib.rs:159`・`:182`・`:186`、`finite.rs:88`・`:93`・`:95`                                                                              | `Value`                                                                                         |
| CheckHistory・ReplayHost・独立のHostMessage                                            | `lib.rs:175`・`:180`、`history.rs:8`・`:9`・`:20`、`host.rs:9`・`:10`・`:24`                                                            | `Value`                                                                                         |
| `Message::Safety`・`Host`・`Results`・`Value`・`Effect`                                | `lib.rs:221`・`:225`・`:226`・`:228`・`:229`、ConformanceResult `:97`・`:98`・`:99`、Effect `:112`・`:113`・`:117`・`:120`、State `:47` | `Value`。SafetyはSafetyResultの行と同じ                                                         |
| core内部のSession・Phase・PreparedFiniteModel・SimulationHost・Operation・ShrinkResult | `lib.rs:237`・`:239`・`:240`・`:246`–`:250`、`finite.rs:9`、`host.rs:56`–`:60`、`history.rs:43`・`:45`、`shrink.rs:46`                  | `Value`                                                                                         |

表のどの辺も、破棄時に入力の深さに比例する標準Dropの再帰を残さない。rtlc・rtla-compiler・idealis-coreで`Box<Self>`・`Vec<Self>`を持つのは`ir.rs`だけで、名前を介した再帰型はIRの型と`Value`だけである。idealis-coreの型はValue以外に深さに比例する所有辺を持たず、`Message::History`・`Candidates`とString・scalarだけの要求・応答には再帰所有辺がない。rtla-compilerの型付きIRはprivateで、rtlaのnesting上限64の中でだけ作られる。

範囲外として残るもの。`Value::normalize`（value.rs:104）、`Arena::data`（safety_values.rs:377）、深い値の`Clone`・`PartialEq`・`Hash`・`Debug`・`Serialize`は破棄ではない別の再帰で、変えない。たとえばversion 1の`Canonicalize`に深いValueを渡すとnormalizeの中で溢れる。このnoteの残課題として記録し、今回のテストはnormalizeより前に拒否される経路を使う。wireのValueはserde_jsonの再帰上限128を受ける。公開の再帰型を持つrtlaのASTは対象外で、そのparserはnesting 64で拒否する。

### テスト計画

テストを先に書く。b5の凍結64ファイルはSHA-256が実装の前後で一致し、56件（rtlcの`deep_drop` 54件、idealis-coreの`deep_request_drop` 1件、rtlc `--lib`の`public_program_deep_action_frames_are_checked_before_recursive_analysis` 1件）は変えない。既存テストの編集は`crates/idealis-core/tests/host.rs:31`のpattern形1か所だけで、テスト段で行う。新しいテストは次の3ファイルに置き、テスト段の終わりに`host.rs`とともに凍結リストへ加える（68件）。

深さとstackは2組に固定する。SHALLOWは1000段を8MiBのthreadで、DEEPは200000段を512KiBのthreadで破棄する。再帰の破棄は1段につき少なくとも戻り先と退避値の16 byteを要し、200000段では3.2MB以上なので、frameを小さくした線形再帰でもDEEPで溢れる。定数stackの実装はどちらも通る。SHALLOWは現行の再帰Dropでも通り、構築・assertion・破棄が深さ以外で成立することを示す対照になる。各ケースはmacroでSHALLOWとDEEPの2テストになり、公開APIだけで値を作り、threadがpanicなくjoinすることを要求する。

`crates/rtlc/tests/deep_drop_paths.rs`（16ケース、32テスト）。

- 単独の破棄（9ケース）: `Not`の鎖の`Expression`、`frame_chain`から取り出した1つの`Branch`、深いbranchesを持つ1つの`Step::Frame`、`Not`の鎖の`TemporalFormula`、`Value`の`List`・`Set`・`Record`・`Map`のkey・`Map`のvalueの各鎖。どれも`SafetyProgram`へ入れず、`check`も呼ばずに破棄する。
- 根の中のfieldの置換・消去（5ケース）: `definitions[0].body`への`truth()`の代入、`next`へ入れたframe鎖の`clear()`、`let Expression::Data { value } = &mut …body else { panic!() }`を通した`*value = Value::Null`、Dataの中の`Value::List`の`clear()`、`std::mem::replace(&mut properties[0].formula, atom())`で取り出した旧formulaの`drop`。代入・clearは旧値をその場で破棄し、その後に浅くなった根も破棄する。
- IRの外のValue（2ケース）: `Scalar::try_from(深いList)`が`("protocol", "Safety values must be integers or booleans")`で拒否されること、深いValueを`SafetyTraceStep.values`に持つ`SafetyResult`の破棄。

`crates/idealis-core/tests/deep_value_drop.rs`（12ケース、24テスト）。

- `Engine::handle`の拒否後にライブラリ内で破棄される要求（7ケース）: version 0の`Canonicalize`（`"Unsupported protocol version"`）、未知handleの`StartLoaded`（actions）・`CheckLoaded`（Traceのinitialとstep）・`CheckHistory`（ConcurrentHistoryのinitialとInvoke）（`"Unknown model handle"`）、version 0の`LoadModel`（Initial.observationとEdge.action）・`ReplayHost`（HostEventのinput）、sessionのない`Resume`（`Response::Observation`、`"No active session"`）。どれもcodeが`"protocol"`のMessage::Errorを要求し、normalizeやprepareより前に拒否される。
- 呼出し側が持つ値（5ケース）: `Message::Value`、`Message::Host`、`Message::Results`（ConformanceResultのactual・expected・trace）、`Message::Effect`（ModelStepのstatesとaction）、独立の`HostMessage.payload`の破棄。
- `Start`・`Replay`・`StartFinite`・`CheckFinite`・`Message::Safety`は、同じ所有型（`Vec<Value>`、Trace、FiniteModel、rtlcのSafetyResult）を上で覆うので足さない。

`crates/rtlc/tests/drop_allocations.rs`（8テスト）。このファイルだけを専用のtest binaryとし、`std::alloc::System`を包む計数用の`#[global_allocator]`を置く。計数はthread localの定数初期化`Cell`で、生存block数、生存byte数、確保回数（alloc・realloc）を測定threadごとに数える。他のテストthreadの確保を数えないので、通常のharnessで並列に走らせてよい。unsafeはこのtest binaryの`GlobalAlloc`実装にだけあり、`System`へ委ねる。各測定は1つのthreadの中で、構築前・構築後・破棄後の計数を読む。

- 鎖の解放（5テスト）: `Expression`の全切り離し規則（Frame・Call・Box・Vec・record・Update・Binding/Domain・Fold・Subseq・If・In・Enabledのbranchesとsubscript・Data）を段ごとに巡る鎖、`Step::Frame.branches → Branch.steps`の鎖、`TemporalFormula`の全variantとAtom・arguments・captures・boundsを巡る鎖、`Value`のList・Set・Record・Mapのkey・valueを巡る鎖、深いbody・next・property・Dataを持つ`SafetyProgram`。SHALLOWとDEEPの両方で、構築が深さ以上のblockを確保したこと（計数器が働くこと）、破棄後の生存block数とbyte数が構築前と等しいこと、破棄中の確保回数が両方の深さで等しいこと（nodeごとや深さに比例する確保がないこと）を要求する。子をforgetする実装は生存数で、辺を切り離し忘れる実装は深さに比例して入れ子になるDropのstackか確保回数で、nodeごとに確保する実装は確保回数で失敗する。
- leakの負対照（2テスト）: DEEPの`Value::List`の鎖と`Not`の鎖で、根の直接の子を`mem::replace`で取り出して`std::mem::forget`し、根を破棄する。計数が「構築したblock数 − 1」（根自身のbufferまたはBoxだけが解放される）のleakを報告することを要求する。故意にleakする破棄を計数が検出することの確認で、実装の前後とも通る。
- 確保なしの破棄（1テスト）: 4型それぞれの、直接の子がすべて葉の値（scalarのrecord、`Add`の2つのLiteral、Guardだけを持つBranch、2つのAtomの`And`）の破棄で、確保回数0と生存数の差0を要求する。Valueの頻繁な破棄で確保しない分岐を守る。

テスト段の受入れ。56件と新しい64件を1件ずつ別process・外側timeoutで実行する。意図したRedは33件で、deep_drop_pathsのDEEP 16件、deep_value_dropのDEEP 12件、drop_allocationsの鎖5件（DEEPの測定で溢れる）である。どれも修正前の標準Dropでstack overflowのabortになり、panicによる失敗ではないことを確かめる。修正前から通るのは31件で、SHALLOWの28件、負対照2件、確保なし1件である。56件は従来どおりabortし、`host.rs`のテストは編集後も通る。

実装段は凍結した68ファイルを変えない。56件と新しい64件を1件ずつ実行し、それぞれ`1 passed`とexit 0を要求する。続いて次をGreenに保つ。Node analysis parity 4236/4236、Rust parity（`--test analysis_parity`）79/79、operator-sharing 970/970、frame decoder 153/153、v1 identity 257/257、`just safety-test`、`just compiler-test`（negate/combineを含む）、rtlcとidealis-coreのfmt・clippy `-D warnings`・`cargo test --all-targets --locked`、`just cross-test`（shared-oracle-rustのfmt・clippy・contract）、`just usable-test`、`just async-test`、`just portable-build`と`just portable-test`、`just criterion-test`、`just check`、`just browser`、`git diff --check`。all-targetsはfilter引数なしで実行する。b5のrtlc all-targetsのexit 101は、`--skip`をharness=falseのcriterion benchへ渡した失敗で、テストの失敗ではない。`typed_expr.rs:96`のCopy束縛がDrop後もcompileすることはclippy・testのbuildで確かめる。criterion `safety`と、Valueを多用するfinite model・conformanceを測る`just usable-benchmark evals/runs/0005-usable-confirmation <新規外部directory>`の前後を同じ機械で比べ、差をそのまま記録する。`just support-plan`を確認し、support-syncのAPI key不在によるskipと既知のstaleなsupport-checkは成功に数えない。有限のstack試験と計数だけで定数stackを証明したとは扱わず、実装レビューで切り離し表の全所有辺（Update.path、Binding.domain、recordの名前、Mapのkey/value、`Option<Box>`、`Vec<Branch>`、Valueの4容器）の処理と、detach後に葉だけが残る不変条件を確かめる。

### P2-1の文言

`docs/rust-safety.md`の75行にある「呼び出し位置で body を展開し、bound slot を2つ使う。」を、`docs/rust-frontend.md`の58行と同じ「foldの専用frameでbound slotを2つ使い、v1では呼び出し位置でbodyを展開する。v2では同じslotをactualとして共有expression定義をcallする。」へ置き換える。段落の他の文は変えない。

## step 3f-b7: 有界stack破棄の先行テスト

3f-b6設計の第2版が前提レビューでGOになったため、本番ソースを変えずに先行テストを追加した。新しいテストは3ファイル64件で、どれも公開APIだけを使う。`crates/rtlc/tests/deep_drop_paths.rs`は16ケースを`shallow`と`deep`の2テストにした32件、`crates/idealis-core/tests/deep_value_drop.rs`は12ケースの24件、`crates/rtlc/tests/drop_allocations.rs`は専用のtest binaryに計数用の`#[global_allocator]`を置く8件である。深さとstackは設計どおり2組に固定した。SHALLOWは1000段を8MiB、DEEPは200000段を512KiBのthreadで構築・破棄する。ケース名はmoduleに置き、テスト名は`<ケース>::shallow`・`<ケース>::deep`になる。既存テストの編集は、ownerが承認した`crates/idealis-core/tests/host.rs:31`の`let Value::Int(index) = &choice else`だけで、assertionは変えていない。

deep_drop_pathsは、`Not`の鎖の`Expression`・`frame_chain`から取り出した`Branch`・深いbranchesを持つ`Step::Frame`・`TemporalFormula`・`Value`のList/Set/Record/Mapのkey/valueの単独破棄9ケース、`definitions[0].body`への代入・`next`の`clear()`・`Data.value`への`Value::Null`の代入・Data内`Value::List`の`clear()`・`mem::replace`で取り出した旧formulaの破棄の5ケース、`Scalar::try_from(深いList)`の拒否と`SafetyResult`の`SafetyTraceStep.values`の破棄の2ケースである。deep_value_dropは、version 0の`Canonicalize`・`LoadModel`・`ReplayHost`、未知handleの`StartLoaded`・`CheckLoaded`・`CheckHistory`、sessionのない`Resume`を`Engine::handle`へ渡して`protocol`の拒否Messageを要求する7ケースと、`Message::Value`・`Host`・`Results`・`Effect`、独立の`HostMessage.payload`の破棄5ケースである。drop_allocationsは、全切り離し規則を段ごとに巡る`Expression`・`Step::Frame.branches`・`TemporalFormula`・`Value`の鎖と、4つの深い鎖を持つ`SafetyProgram`の5件、DEEPの`Value::List`と`Not`の鎖で根の直接の子を`mem::replace`で取り出して`std::mem::forget`し、計数が構築block数−1のleakを報告することを要求する負対照2件、直接の子がすべて葉の4型の値の破棄で確保回数0と生存数の差0を要求する1件である。鎖の5件は、SHALLOWとDEEPの両方で構築が深さ以上のblockを確保したこと、破棄後の生存blockとbyteが構築前と等しいことを要求し、さらに破棄中の確保回数が両方の深さで等しいことを要求する。

鎖の各段は葉でない子を1つだけ持たせた。初版の鎖は段ごとに葉でない兄弟を持たせたため、worklistへ積まれた兄弟が鎖の処理が終わるまで残り、最大長と再確保回数が深さに応じて増えた。作業木の外で試作した有界Dropは3件が「破棄中の確保回数が深さで異なる」と失敗したので、兄弟を葉だけに直し、`Data`と非葉の`Atom`は鎖の終端に1回だけ置いた。設計の「各nodeの葉でない子が1つの鎖ではworklistの長さが定数に留まる」の前提を、鎖の形で守る変更で、設計は変えていない。

DEEPのstackが固定frameの実装で足りることは、全64件の構築と破棄をSHALLOWの深さ1000段・512KiBのthreadで実行して確かめた。作業木の外に置いた一時の複製で、DEEPの深さを1000へ、SHALLOWのstackを512KiBへ変えると、各ファイルの全テストが同じ形で1000段・512KiBに揃う。drop_allocationsの2つの深さが同じ1000段になるので、確保回数の比較は自明に等しい。結果は、deep_drop_paths 32/32、deep_value_drop 24/24、drop_allocations 8/8の計64/64が現行の再帰Dropのまま通った。drop_allocationsの鎖のbuilderは、破棄をforgetに替えたDEEPの構築だけでも200000段・512KiBで溢れずに完了した。試作の有界DropでDEEPの全テストが通るので、DEEPで溢れる原因は構築ではなく破棄の再帰である。

修正前のRedは33件で、すべてstack overflowのabort（exit −6）であり、panic表示がない。deep_drop_pathsのDEEP 16件（`data_list_clear`・`data_value_assignment`・`definition_body_assignment`・`next_branches_clear`・`property_formula_replacement`・`safety_result_trace_values`・`scalar_refusal`・`standalone_branch`・`standalone_expression`・`standalone_step_frame`・`standalone_temporal_formula`・`standalone_value_list`・`standalone_value_map_key`・`standalone_value_map_value`・`standalone_value_record`・`standalone_value_set`の各`::deep`）、deep_value_dropのDEEP 12件（`refused_canonicalize_unsupported_version`・`refused_start_loaded_unknown_handle`・`refused_check_loaded_unknown_handle`・`refused_check_history_unknown_handle`・`refused_load_model_unsupported_version`・`refused_replay_host_unsupported_version`・`refused_resume_without_session`・`owned_message_value`・`owned_message_host`・`owned_message_results`・`owned_message_effect`・`owned_host_message_payload`の各`::deep`）、drop_allocationsの鎖5件である。drop_allocationsの鎖5件は、SHALLOWの測定とassertionが通った後、DEEPの破棄で溢れる。修正前から通るのは31件で、各ケースのSHALLOW 28件（再帰Dropでも1000段は8MiBに収まるため）、負対照2件（根の直接の子を取り除くため再帰しない）、確保なし1件（直接の子がすべて葉で、現行のDropも確保しない）である。既存56件（rtlcの`deep_drop` 54件、idealis-coreの`deep_request_drop` 1件、rtlc libの`public_program_deep_action_frames_are_checked_before_recursive_analysis` 1件）は従来どおりすべてstack overflowでabortし、`host.rs`のテストは編集後も通る。

テストが満たせることの確認として、作業木の外の複製に4型の有界Drop（worklist1本）と、`normalize`・`negate`・`combine`の移行を試作して新64件と既存56件を実行した。新64件と、既存のrtlc 54件・core 1件はすべて通過した。これは設計の実装ではなく、本番ソースへの反映もない。実装段で本番の実装を書き、同じ凍結テストを通す。

Greenを保ったのは、Node analysis parity 4236/4236、Rust parity全manifest 79/79（rtla-compiler 28件とrtlc lib 51件を1件ずつ別process）、operator-sharing 970/970、frame decoder 153/153、v1 byte identityとbaseline 257/257、`just safety-test` 1183/1183、`just compiler-test`、`just check`、rtlcとidealis-coreのfmtとclippy `-D warnings`（新しいテストを含む）、red 56件と新Red 33件を除くrtlcとidealis-coreのテスト、criterion benchの`safety`、`git diff --check`である。

凍結リストはb5の64件に`host.rs`と新規3ファイルを加えた68件で、`analysis-inventory.json`は引き続き除外する。本番ソース、manifest、fixture、既存TLCログ、v1 baselineは変更せず、全差分は未コミットのまま、stage/unstage・履歴/refs・GitHub書込み・新TLC採取・mutationは行っていない。

## step 3f-b8: 有界stack破棄の実装

3f-b6設計の第2版に従い、公開IRと`Value`の破棄を有界stackにした。新しいprivate module `crates/rtlc/src/bounded_drop.rs`（`lib.rs`に`mod`を1行追加）に、非葉の所有子だけを運ぶ`Detached`（Expression・Branch・TemporalFormula・Valueの4 variant）とworklist（`Vec<Detached>`）、4型の葉判定、`push_*`と`detach_*`、4つの`Drop`を置いた。4つのDropは同じ`release`を呼ぶ。`release`は`Worklist::new()`へ自分の非葉の子だけを移し、空になるまでpopして同じ`detach_*`を当てる。Box内は`mem::replace`で`Expression::Old`・`Atom { Old }`と、Vec・record・Mapは`mem::take`、`Option<Box>`は`take()`、`Data.value`は`Value::Null`との置換で切り離し、取り出した葉はその場で破棄する。popしたnodeの破棄で走るDropは、直接の子がすべて葉なのでworklistを確保せず戻る。`detach_expression`・`detach_formula`・`detach_value`・`detach_step`・`detach_domain`はワイルドカードを持たないmatchなので、variantの追加は構築時の失敗になる。`is_leaf_expression`の`_`は非葉への既定で、追加variantを葉とは扱わない。設計表との違いは書き方だけで、`Step`・`Binding`・`Domain`は所有するVecを`mem::take`したうえで取り出した要素へ`&mut`のdetachを当て、`Update`は値で分解する。公開型のfield・variant・serde属性・derive・methodは変えていない。

migrationは設計の6関数で、コンパイラが示したE0509は`value.rs`の`normalize`（7箇所）、`counter.rs`（2箇所）、`shared-oracle-rust`の`native_value`（3箇所）で、`negate`と`combine`は設計表どおり先に移した。`normalize`は`mut self`と`match &mut self`にしてString・Vecを`mem::take`し、`Null | Bool | String`は`Ok(self)`にした。`negate`と`combine`は`&mut`で照合して式を`mem::replace(.., Old)`で取り出し、`combine`の他の組は`_`で元の`left`・`right`を包む。`counter.rs`は25行を`&entry.state`、65行を`match &action`、`native_value`は`mut value`と`match &mut value`にした。`host.rs`はb7で変更済みである。移行後はrtlc・rtla-compiler・idealis-core・shared-oracle-rustの全targetの`cargo check`とidealis-coreのwasm32 lib checkが通る。`crates/rtlc/README.md`の`rtlc::ir`の段落、`crates/idealis-core/README.md`の再exportの文へ、`Drop`の実装とmove不可・参照／`std::mem::take`／`std::mem::replace`での取り出しを1文ずつ加え、`docs/rust-safety.md`の75行をP2-1の文言へ置き換えた。

意図したRedは全てGreenになった。各テストを別processと60秒の外側timeoutで実行し、全件が`1 passed`・exit 0だった。新Red 33件はdeep_drop_pathsのDEEP 16件、deep_value_dropのDEEP 12件、drop_allocationsの鎖5件、既存Red 56件はrtlcの`deep_drop` 54件、idealis-coreの`deep_request_drop` 1件、rtlc libの`public_program_deep_action_frames_are_checked_before_recursive_analysis` 1件である。修正前から通っていた31件（SHALLOW 28件、負対照2件、確保なし1件）も通ったままで、計120件に回帰はない。leakの負対照は計数の報告どおり構築block数−1を検出し続け、鎖の5件は深さ1000と200000で破棄中の確保回数が等しい。

分析のinventoryは1回だけ追随が要った。`typed_lower.rs`の変更でNode analysis parityが1件、`analysis inventory source digest changed`で失敗した（4236中4235）。b3・b4と同じく凍結extractorの`extractAnalysisInventory()`だけで`extracted`を再生成した。変わったJSON pathは`/extracted/sourceSha256/crates/rtla-compiler/src/typed_lower.rs`の1件で、`extracted`以外の10 top-level fieldsは不変、専用guardはexit 0だった。再実行で4236/4236になった。そのほかの結果は次のとおり。Rust parityは79/79（rtla-compiler 28件とrtlc lib 51件を1件ずつ別process）、operator-sharing 970/970、frame decoder 153/153、v1 byte identityとbaseline 257/257、`just safety-test` 1183/1183、`just compiler-test`（rtla-compilerのfmt・clippy `-D warnings`・97 tests）、rtlcとidealis-core・rtla-compiler・shared-oracle-rustのfmt、rtlcとidealis-coreのclippy `-D warnings`、フィルタ引数なしの`cargo test --all-targets --locked`はrtlc 207件（lib 104）・idealis-core 43件、`just portable-build`、`just portable-test`（実Chromium 2件）、`just criterion-test`（criterion14本のsmokeとcontract 1件）、`just check`（Vitest 77とnode:test 77）、`just browser`（実Chromium 3件）、`just usable-test` 4/4、`git diff --check`、shared-oracle-rustのclippy `-D warnings`である。`just support-plan`は7 crates/28比較、`just support-sync`はAPI key不在の警告skip、`just support-check`は既知のstaleでexit 3のままであり、同期や鮮度検査の成功とは扱わない。support記録・digestは手編集していない。

`just cross-test`と`just async-test`はこの機械で完了しなかった。どちらも前提の`cross-build`にある`go build`が`go: command not found`でexit 127になり、テストまで進まない（Goは未installで、installは行っていない）。Goを要しない部分として、shared-oracle-rustのfmtとclippyは成功し、release binaryはbuildできた。`native_value`の変更を確かめるため、リポジトリ外へ置いた`executeSuite`の写しでGoの脚をRustの出力に置き換え、TSとRustだけを比べた。19 case×6 faultで両者のdigestが一致し、runnerの全assertion（通常対照・不一致・呼出し回数・finiteとreplayの一致）が成立した。リポジトリのcontract testは`.artifacts/go`の不在でENOENTになる（cross 0/2、async 1/4）。これはGoを含む脚を実行していないという記録で、成功とは数えない。Goのある環境での再実行が残る。

計測は同じ機械で、他のセッションの負荷（load average 1.9〜3.7）がある中、判定ではなく数値として記録した。criterion `safety`（`cargo bench --bench safety`、14本、埋込fixtureの入力、各側2回）は、2回の中央値の平均の後/前が0.978〜0.998（幾何平均0.989）で、run間のばらつきは前が最大1.019、後が最大1.045である。`just usable-benchmark`（前2回・後4回、各batchの中央値、ms）は、size 64でwasm-wireが3.44・3.40から3.68・3.74・3.66・3.68（平均+7.8%）、native-jsonlが2.66・2.59から2.74・2.92・2.62・2.68（平均+4.4%、run間のばらつきは最大1.11）、Rustを通らないts-preparedは2.47・2.38から2.37・2.65・2.42・2.39、ts-wireは5.21・4.75から4.82・5.19・4.95・4.92、size 16でwasm-wireが1.24・1.22から1.30・1.32・1.30・1.32（平均+6.4%）、native-jsonlが0.92・0.88から0.92・0.92・0.90・0.92である。size 1は前の1回目が冷えた実行で、native起動も272msの外れ値のため比較に使わない。wasm-wireの増加はこのbenchmarkのrun間のばらつきを超えており、Valueが入れ子の容器を持つ破棄で外側1回につきworklistを確保する設計どおりの費用と整合する。設計の葉の定義は変えていない。

実装の範囲外として残るもの。`Value::normalize`、`Arena::data`と、深い値の`Clone`・`PartialEq`・`Hash`・`Debug`・`Serialize`は破棄ではない別の再帰で、変えていない。たとえばversion 1の`Canonicalize`に深い`Value`を渡すとnormalizeの中で溢れる。有限のstack試験と計数だけで定数stackを証明したとは扱わず、実装レビューで切り離し表の全所有辺と、detach後に葉だけが残る不変条件を確かめる。凍結68ファイルは開始時・終了時とも68/68 OKで、変更したのは本番source・README・文書・inventoryのextracted・この記録で、テストと凍結ファイルには触れていない。stage/unstage・commit・履歴/refs・GitHub書込み・新TLC採取・mutation・依存変更・limit引上げ・code comment追加は行っていない。

## step 3f-b9 design: 通常の値に費用を足さない有界stack破棄

改訂: 2026-11-17 初版（前提レビュー前の草案）。

第七回最終レビューはGOだったが、b8の破棄はusable benchmarkのwasm-wireをsize 16で6.4%、size 64で7.8%遅くした。ownerはこのpull requestでこれらのbenchmarkを遅くしないことを求めている。この節は増えた時間の出所の測定、通常の値で費用を足さない設計、測定前に固定する受入れ基準、テスト計画を決める。本番ソースとテストは変えていない。測定と試作は作業木外の複製（`<scratch>/step3f-b9-design/`）で行い、数値は同じ場所の`records/measurements-summary.json`に記録した。

### 測定: 増えた時間の出所

before側は作業木からb8の破棄変更だけを外した複製である。`crates/rtlc/src/bounded_drop.rs`と`lib.rs`の`mod`行を除き、`value.rs`をHEADの`normalize`に戻した。`typed_lower.rs`・`counter.rs`・`native_value`の移行はDropがなくてもcompileし、測定経路の外なので両側で同じにした。usable benchmarkの各batchが送る`check-finite`コマンド（`evals/runs/0005-usable-confirmation`のmodelと固定history、size 1・16・64）を保存し、idealis-coreと同じrelease profile（LTO、codegen-units 1、panic abort）でlinkした計測用binaryで`Engine::dispatch`と出力のserializeを実行した。

| 測り方                                                                 | before                                     | b8                                       | 読み                                                                                                                                                                                      |
| ---------------------------------------------------------------------- | ------------------------------------------ | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 計数allocator（1要求、size 1/16/64）                                   | 7,592 / 21,977 / 68,009回                  | 7,804 / 22,939 / 71,371回                | 差は212 / 962 / 3,362回。reallocは同数で、増えたbyteは1回352 B（88 Bの`Detached`×初期容量4）。非葉の子を持つ`Value`の破棄1回につきworklistを1回確保している                               |
| 退役命令数（`/usr/bin/time -l`、run間差0.5%以下）                      | 基準                                       | +5.2% / +9.8% / +11.9%                   | 時間（cycles）もほぼ同じ比で増える                                                                                                                                                        |
| sampling（`sample`、size 64、各側1回約5,800 sample）                   | `Value`破棄の包含時間9.3%（約120 µs/要求） | 21.6%（約315 µs/要求）                   | 増分約195 µsのうち、allocatorのself時間が約+68 µs（35%）、bounded_dropの自前code（`release`のclosure、`detach_value`による全子の値渡し走査、88 Bの`Detached`のpush/pop）が約+86 µs（44%） |
| wasm、同一process内でbeforeとb8を交互に600回ずつ`checkFiniteHistories` | 基準                                       | size 16で+5.9〜6.8%、size 64で+7.2〜7.8% | usable benchmarkのwasm-wireの差と一致する                                                                                                                                                 |

size 64の1要求で破棄される`Value`は約31,400個で、そのうち非空の容器が10,585個、非葉の子を持つ容器（主にobservationの`{granted: List, locked}`）が3,362個である。退行の原因はworklistの確保だけではない。確保は増分の約4割で、残る約半分は、葉や葉だけを子に持つ容器も含めたすべての`Value`の破棄が`release`を通り、子を一つずつ値で取り出してから破棄する固定費である。Dropなしで`value.rs`の`normalize`移行だけを当てた複製は命令数+0.4%で、そこへ空の`Drop`を足した差は±0.3%以内だった。Dropを置くこと自体の費用は小さい。

### 比較

各案を同じ計測binaryで試作し、命令数とwasmの同一process比較で測った。

| 案                                                                                                      | 命令数（size 64）                                  | wasm-wire（同一process比較） | 判断             |
| ------------------------------------------------------------------------------------------------------- | -------------------------------------------------- | ---------------------------- | ---------------- |
| (a1) 予算付き再帰。子を`&mut`で走査し、非葉の子は再帰の後に葉と置換して破棄する。gateは自分の葉判定     | +3.9%                                              | +1.6〜2.8%（usable実行）     | 不採用           |
| (a2) (a1)の走査を値渡しへ融合                                                                           | +2.9%                                              | 未測定                       | 不採用           |
| (a3) b8の切り離し関数に予算を足し、`Value`のgateを「直接の子がすべてflat（高さ2以下）」にしてglueへ委任 | +1.6〜1.7%                                         | +0.5〜1.4%                   | 採用の骨格       |
| (a4) (a3)＋`normalize`のin-place化                                                                      | −0.0〜−0.1%（size 16は−0.4〜−0.5%、size 1は−0.7%） | +0.4〜1.0%                   | 採用             |
| (a4)からinline指定を外したもの                                                                          | +1.6%                                              | 未測定                       | 指定は設計の一部 |
| (b) inline small buffer（stack配列から溢れたらheap）                                                    | 試作せず                                           | 試作せず                     | 不採用           |
| (c) thread localの再利用worklist                                                                        | 試作せず                                           | 試作せず                     | 不採用           |

(a1)・(a2)は確保を消したが、置換した旧nodeのDropが子を再走査し、非空の容器ごとに`release`への呼出しと値の移動が残った。(b)はb8の分解のうちallocator分（約4割）しか消さない。`[Option<Detached>; N]`をreleaseのframeに置くので、全`Value`の破棄が通るreleaseの費用はむしろ増える。(c)も確保を初回だけにするだけで固定費は残り、さらに次の危険を持つ。popしたnodeのDropが同じworklistを再び借りる再入（外側の借用を解いてから破棄するか、入れ子では局所Vecへ逃がす分岐が要る）、thread終了時のTLS破棄順（他のthread localが保持する`Value`をworklistのTLSより後に破棄すると`with`がpanicし、`try_with`の失敗時に局所Vecへ戻す分岐が要る）、深い破棄の後に容量を保持し続けること、global mutable stateそのものである。wasm32の単一threadでは静的変数になるだけで安全だが、nativeのrayon workerではthreadごとに容量を持つ。

(d) その他。stacker型の残りstack検査はplatformのstack境界をunsafe FFIで読む必要があり、wasm32ではshadow stackとengineのnative stackが別で使えない（3f-b6の棄却理由と同じ）。親のVecへ孫を移して平坦化する方式（pointer reversal系）は同じ型の容器（`List`・`Set`）にしか効かず、`Record`・`Map`のtupleや4型の循環を扱えず、親Vecの再確保を生む。高さ検査だけでglueに任せ、超えたらworklistへ送る方式は、高さ3以上の通常の値で確保が残る。`Value`だけを変える方式は不変条件が型ごとに分かれるので、予算の仕組みは4型に共通とし、gateだけを型ごとに置く。

### 採用: 予算付き再帰、Valueの高さ2までのglue委任、normalizeのin-place化

`bounded_drop.rs`の変更は次のとおり。

- 定数`DEPTH_BUDGET: usize = 32`を置く。
- すべての`detach_*`（`detach_boxed`・`detach_boxed_formula`・`detach_domain`・`detach_bindings`・`detach_step`・`detach_branch`・`detach_value`・`detach_expression`・`detach_formula`）が`budget: usize`を引数で受ける。予算はこの引数だけで運び、global・thread local・`Detached`のfieldには持たない。
- `push_*`を`dispose_*`に置き換える。切り離した子は、(i) 葉なら（`Value`はflat、つまり直接の子がすべて葉なら）その場で破棄する、(ii) `budget`が0ならworklistへ積む、(iii) それ以外は同じ`detach_*`を`budget − 1`で呼び、子の子をすべて切り離してから破棄する。
- `release`はworklistからpopしたnodeを`DEPTH_BUDGET`で`detach_*`する。popは根のDropの反復の中で行い、入れ子にならない。
- `Drop for Expression`・`Branch`・`TemporalFormula`は、自分が葉なら何もせず戻り、そうでなければ`release`を`DEPTH_BUDGET`で始める。
- `Drop for Value`は`#[inline]`で、`is_shallow_value`（直接の子がすべてflat）なら何もせず標準のdrop glueに任せる。そうでなければ`#[inline(never)]`の`release_value`へ入る。
- `is_leaf_value`・`is_flat_value`・`is_shallow_value`・`dispose_value`は`#[inline(always)]`、`Value`の再帰段`descend_value`は`#[inline(never)]`とする。指定を外した試作は命令数+1.6%で、gateがdrop glueへ展開されることが費用の前提である。
- `Detached`・`Worklist`・葉判定・3f-b6の切り離し表の全所有辺は変えない。公開型、README、`check`の結果も変わらない。

`value.rs`の`normalize`は、b8の`mem::take`による移行をin-placeに改める。`ModelValue`・`Int`・`Bytes`は参照で検査して`Ok(self)`を返す。`List`は`iter_mut`で各要素を`Value::Null`と置換して正規化し書き戻す。`Set`・`Record`・`Map`は`drain(..)`でBTreeMapへ移し、同じVecへ`extend`で戻す。出力・エラー・評価順は同じで、容器を正規化するたびのVec確保が1回減る（size 64で1要求68,009回→66,052回）。挙動は`idealis-core/tests/contract.rs`の共有fixture（valid 7件、重複record・重複map・非正規整数・不正bytesを含むinvalid 15件）と既存suiteが覆う。

不変条件。再帰は1段ごとに予算を1減らし、worklistへ積むのは予算0のときだけである。破棄の途中で走るDropは、子をすべて切り離されたIR node（`release`が子の葉判定だけで戻る）、flatまたはshallowな`Value`（drop glueが高々3段）、空になった容器の3つだけで、どれも定数段で終わる。入れ子のDropが新しい予算で深く再帰することはない。時間はnode数に線形で、`Value`のgateは各nodeを高々3回読む。確保は予算を超えたときだけで、根のDrop1回につきworklist1本の償却拡張である。予算の範囲は、凍結した確保回数の一致からSHALLOWの1000段未満、DEEPの512 KiB（debugで1予算約2.9 KiB）から約170以下である。32はreleaseのstackを約32 KiBに抑え、finite modelの値（高さ2以下）はtraversalにすら入らない。受理されるIR（入れ子48まで）は予算を超え得るが、programの破棄は1回なので、確保は根ごとにworklist1本に留まる。

試作(a4)は凍結テストを変えずに、rtlcの`cargo test --all-targets`207件（lib 104、deep_drop 54、deep_drop_paths 32、drop_allocations 8）、idealis-coreの43件（deep_value_drop 24、deep_request_drop 1を含む）、rtlcとidealis-coreのclippy `-D warnings`、rtlcのfmtを通った。

### stackの上限

上限は予算Bだけで決まり、入力の深さによらない。frame数は構造から、byte数はrustc 1.100.0で作った関数のprologueから測った（B=32）。

| 区間                                                                                                               | frame数    | aarch64 release（LTO） | aarch64 debug（テスト） | wasm32 release（shadow stack）  |
| ------------------------------------------------------------------------------------------------------------------ | ---------- | ---------------------- | ----------------------- | ------------------------------- |
| 1予算あたりの最大（IR。`detach_expression → detach_bindings → detach_domain → detach_boxed → dispose_expression`） | 5          | 944 B                  | 2,864 B                 | 368 B                           |
| 1予算あたり（`Value`。`descend_value → detach_value`）                                                             | 2          | 272 B                  | 2,512 B                 | 176 B                           |
| 根（drop glue、`release`）と底（予算0の積込み、空になったnodeのDrop、shallowな`Value`のglue 3段）                  | 8          | 約1.5 KiB              | 約3.7 KiB               | 約0.6 KiB                       |
| 合計の上限                                                                                                         | 5B+8 = 168 | 約32 KiB               | 約96 KiB                | 約12 KiB（wasm frameは168以下） |

b8の定数段より大きいが、DEEPの512 KiB threadではdebugでも5倍以上の余裕がある。

### 凍結したテストが変えずに通る理由

- 89件の深い破棄（DEEP 200000段を512 KiB、deep_dropは100000段を8 MiB）: 再帰は予算で止まり、debugで約96 KiB以下である。
- `assert_chain_freed`の確保回数の一致: SHALLOWの1000段もDEEPの200000段も予算を超えるので、どちらも予算0の点で1回だけworklistを確保する。鎖の各段の非葉の子は1つなので長さは1を超えず、再確保もない。`SafetyProgram`は4本の根でそれぞれ1回である。両方の深さで回数が等しい。
- `nodes_whose_children_are_all_leaves_drop_without_allocation`: scalarのrecordはshallowでglueに任される。2つのLiteralの`Add`、Guardだけの`Branch`、2つのAtomの`And`は子が葉なので何も積まない。確保は0である。
- leakの負対照: 根の子を葉と置換してforgetするので、根は葉の子しか持たず、計数は構築block数−1のままである。

### 受入れ基準（測定前に固定）

before = 作業木からb8の破棄変更（`bounded_drop.rs`、`mod`行、`normalize`の移行）だけを外した複製、after = 新設計を実装した作業木の複製とする（usable benchmarkのsource fingerprintが`git ls-files`を使うので、どちらの複製もgit metadataを持つ）。どちらもこの機械で`just portable-build`を行い、`cargo bench --locked --manifest-path crates/rtlc/Cargo.toml --bench safety -- --save-baseline b9-<側>-<i>`と`just usable-benchmark evals/runs/0005-usable-confirmation <新規外部directory>`を各側N=10回、B A A B B A A B…の順（ABBA×5）で交互に実行する。

pathは22本である。criterionは14本で、各runの値は`estimates.json`の`median.point_estimate`とする。usableはsize 16と64の`ts-prepared`・`ts-wire`・`wasm-wire`・`native-jsonl`の8本で、各runの値は`result.json`の`batches[size].targets[name].medianMs`とする。b8と同じく、起動直後のsize 1と単発の起動時間は比較に使わない。

path pについて、before側の値をb_1..b_10、after側をa_1..a_10とし、S_p = max(b) / min(b)、R_p = mean(a) / mean(b)とする。合格は22本すべてでR_p ≤ S_pであることで、速くなる側は制限しない。加えて、凍結したテストと新しいテストが変わらずGreenで、b8で記録したsuiteがすべてGreenであること。再実行で判定を変えず、同一process比較と命令数は判定に使わない診断として記録する。

事前の試作でこの計算を当てた。(a4)はusable 8本（ABBA、各側8回）ですべて合格し、最も近いのは`16/wasm-wire`のR 1.0124 ≤ S 1.0144だった。criterion 14本（before 8回、(a4) 6回）もすべて合格し、幾何平均1.0006、最も近いのは`values/intern-existing-functions`のR 1.0169 ≤ S 1.0191だった。b8へ同じ計算を当てると（各側9回の交互実行）`16/wasm-wire`・`64/wasm-wire`・`64/native-jsonl`が不合格になり、基準はb8の退行を検出する。

危険が一つある。criterionの`values/intern-*`はScalarだけのarenaを測り、変更した4型を一切破棄しないが、Dropを持つどの版でもbeforeより0.5〜2%遅く出た。codeの配置による差と考えられ、各側4回では不合格になった。N=10はこの揺れをS_pに含めるための回数である。

### テスト計画

凍結68ファイルは変えない。新しいtest binary `crates/rtlc/tests/drop_budget.rs`を一つ足し、drop_allocations.rsと同じ形の計数`#[global_allocator]`（thread localの定数初期化`Cell`、測定threadごと）を置く。各テストは8 MiBのthreadで公開APIだけを使って構築・破棄し、破棄中の確保回数と、生存block・byteが構築前に戻ることを要求する。5件である。

1. `value_chain_up_to_the_depth_budget_drops_without_allocation`: `Value::List`を34段入れ子にした鎖（予算32＋shallowの2段）。確保0。
2. `value_chain_one_level_past_the_depth_budget_allocates_one_worklist`: 同じ鎖の35段。確保ちょうど1。
3. `expression_chain_up_to_the_depth_budget_drops_without_allocation`: `Not`の33段の鎖。確保0。
4. `expression_chain_one_level_past_the_depth_budget_allocates_one_worklist`: `Not`の34段。確保ちょうど1。
5. `ordinary_values_and_programs_drop_without_allocation`: observationのrecord、その`List`、model値から`Set`・`List`を持つrecordへの`Map`（高さ4）、drop_allocationsと同じ形の`value_cycle`・`expression_cycle`・`frame_chain`・`formula_cycle`と深いfieldを持つ`SafetyProgram`の各8段。どれも確保0。

テスト段のRedは、b8の実装で1・3・5が`allocations during the drop`のassertion失敗（left 1、right 0）で落ちることで、abortではない。2・4はb8でも通る（b8はどの鎖でもworklistを1回確保する）。この2件は予算の上側を固定するためのもので、作業木外の試作で`DEPTH_BUDGET`を33にすると2・4が、31にすると1・3が落ちることを確かめた。下書きは`<scratch>/step3f-b9-design/drafts/drop_budget.rs`に置いた。テスト段の終わりにこのファイルを凍結リストへ加え、69件とする。

実装段は凍結した69ファイルを変えない。既存の89件、drop_allocations 8件、新しい5件を1件ずつ別processで実行し、b8と同じsuiteをGreenに保ってから、上の受入れ計算を行う。

### 残すもの

- 試作(a4)でもwasm-wireは同一process比較で+0.4〜1.0%残る。usable benchmarkのrun間差（1.4〜5%）の中だが0ではない。`Value`のgateがdrop glueの前に子と孫を読む費用で、検査してからglueに任せる形ではこれが下限に近い。
- `Value::normalize`以外の深い値の再帰（Clone・PartialEq・Hash・Debug・Serialize、`Arena::data`）はb8の残課題のままである。

## step 3f-b10 design: 受入れ計算の固定、placebo、残差の試作、C2・C3のテスト

改訂: 2026-11-18 初版（前提レビュー前の草案）。3f-b9の前提レビューはNO-GO（C1・C2・C3）だった。3f-b9からの変更と理由は次のとおり。

- 受入れ式`R_p = mean(after)/mean(before) ≤ S_p = max(before)/min(before)`を削除した。beforeの散らばりを許容回帰に変え、全組で1%遅い実装を通し、同速の実装をノイズで落とすため（C1）。代わりに、順序を均衡させたblockごとの対数比と片側上側信頼限界による計算を、新しい測定の前に固定した。
- 同一process比較（in-process）を診断から判定pathへ移した。既知の正の残差を判定の外に置かないため（C1）。
- 挙動を変えずcodeだけを動かすplacebo armを足し、code配置だけの効果を同じprotocolで測った（C1）。
- 残差を消すことを狙う試作を3案作り、同じ計算で測った。
- 浅い非葉の子を所有辺へ1つずつ置くテスト（C2）と、`normalize`のエラー優先順位を正確な`code`・`message`で固定するテスト（C3）を足した。
- stackの上限を、`5B+8`の厳密な導出ではなく、O(B)とsource上の最長経路の段数で述べ直し、byte数はtoolchain・target・profileを限った見積りにした（レビューの限定）。
- b8の費用の「確保が約4割・固定費が約半分」という内訳は、1回ずつのsamplingによる補助証拠であり、厳密な因果の分解ではないと改めた。確保とrelease経路の両方が増分に寄与する、という向きだけを使う（レビューの限定）。
- `normalize`のin-place化は破棄の費用と分けて報告する。in-place化だけを当てたarm（norm）を置き、破棄の費用はnorm比で示す。3f-b9の「容器を正規化するたびVec確保が1回減る」は広すぎたので、確実に言えるSet・Record・Mapの再構築先Vecの再利用と、要求全体の確保回数の差（size 1・16・64で7,592→7,399、21,977→21,364、68,009→66,052）に改めた（レビューの限定）。
- inline属性はcompilerへのhintで、測定値とbyte数はrustc 1.100.0と記したtarget・profileに限ると明記した（レビューの限定）。

本番ソースとテストは変えていない。試作・測定・script・生標本は`<scratch>/step3f-b10-design/`（以下、記録の場所はここからの相対path）に置いた。3f-b9の記録は保存値の再集計にだけ使った。

### 受入れ計算（新しい測定の前に固定）

計算は`scripts/accept.py`（`REGISTRATION`定数）と`scripts/tdist.py`にあり、固定した時刻と内容のsha256を`records/registration.json`・`records/registration.sha256`に記録してから新しい測定を始めた。以後この計算は変えていない。

- 単位。usableとcriterionは鏡像blockを単位とする。blockの中で各armは2回走り、後半は前半の逆順なので、各armの平均位置が等しく、時間に線形な揺れは差から消える。2 armならABBAとBAABを交互に置き、多armでは前半の順序をblockごとに回す。in-processはnode process 1本を単位とする。processの中で各armは600回ずつ走り、順序は1回ごとに往復し、moduleの読込み順はprocessごとに回す。
- runの値。usableは`batches[size].targets[name].medianMs`（size 16と64）、criterionはそのrunのbaselineの`estimates.json`の`median.point_estimate`、in-processはprocess内の600標本の中央値。生標本はすべて保存する（usableは各runのjsonの`samplesMs`、criterionはbaselineごとの`sample.json`、in-processは全600標本）。
- 統計。path pのblock jについて、d_j = mean ln(after) − mean ln(before)（同じblockの中）。n blockの平均をd̄、標準偏差をsとし、U_p = exp(d̄ + t(0.975, n−1)·s/√n)、L_p = exp(d̄ − t(0.975, n−1)·s/√n)とする。片側97.5%で、非劣性検定の慣例（両側95%区間の上端）と同じ水準である。
- 判定path。criterion 14本、usableのwasm-wireとnative-jsonl（size 16・64）4本、in-processのwasm（size 16・64）2本の計20本。usableのts-preparedとts-wire（4本）は両側でまったく同じTypeScriptを測るので、判定に入れず、protocolの対照として同じ計算で記録する。margin 0では等速のpathが通る確率は2.5%なので、変更の及ばないpathを判定に入れると、どの変更も通らなくなるからである。
- 多重性。受理は「判定20本すべてでU_p ≤ 閾値」のときだけである（intersection-union test）。各pathを水準2.5%で検定すれば、全体の誤受理率も2.5%以下で、調整は要らない（Berger 1982）。その代わり受理の検出力はpathごとの検出力の積になるので、標本数はpathあたりの第2種の誤りβ = 1 − 0.8^(1/20) = 1.1%で決める。
- 標本数。保存した3f-b9の記録（usableはu1〜u5の全run、criterionはbefore 8回とproto2 6回、in-processは3 process）からblock値の標準偏差を推定し、その80%上側信頼限界（χ²で自由度を補正）を使った（`records/pilot.json`）。見つける差Δは、in-processで0.4%（3f-b9の(a4)の最小残差+0.37%を解像する）、usableとcriterionで1%（run間の散らばりが1.4〜4.2%あり、0.4%には150〜1,371 blockが要る。1%は3f-b9がcriterionで見た配置の揺れの大きさ）。n = min{n : (t(0.975, n−1) + t(1−β, n−1))·σ/√n ≤ ln(1+Δ)}をpathごとに求め、familyの最大を取った。usable 129 block、criterion 40 block、in-process 27 processである。
- 結果の種類。全判定pathがnot-slowerならaccept、どこかでL_p > 閾値ならslower、それ以外はinconclusive。inconclusiveはGreenにしない。block数が登録値と違う、runが欠ける、blockの釣り合いが崩れる、値が正の有限数でない、のどれかなら計算を拒否する。再測定で判定を変えない。
- 自己検査（`scripts/accept_selftest.py`、8/8）。レビューの反例（a = 1.01·b）と全組1%遅い系列はacceptにならない。等速の40系列でslowerは3以下、inconclusiveは35以上。2%速い系列はacceptになる。runの欠落、登録より少ないblock、0の値は拒否される。

### 余裕幅の選択肢と推奨

閾値は2通りあり、どちらも同じblock・同じnで測ったafterのU_pと比べる。

- margin 0: 全判定pathで閾値1。許容の根拠を必要としない。
- placebo margin: path pの閾値をmax(1, U_p(placebo))とする。placeboはafterと同じblockで同じ回数測る。「挙動を変えない変更と区別できない」ことを受入れの意味にする。

測定の前に分かる帰結。margin 0では、真の比が1のpathを通る確率は2.5%以下で、変更の仕事が及ばず配置だけが変わるpath（criterionの多くは計測区間で4型を破棄しない）があるので、等速の変更も挙動を変えない変更もほぼ通らない。受理されるのは全判定pathで速いと示せる変更だけで、全pathでΔだけ速い変更は80%以上で受理される。placebo marginでは、placeboと同じ分布のafterでも各pathを通る確率は約50%である（同じnと散らばりなら、U_pの大小はd̄の大小と同じ）。20本すべてなら独立として0.5^20で、やはりほぼ通らない。さらに閾値はplacebo系列の雑音とともに大きくなるので、雑音を許容幅に変えるというC1の欠陥を一部持ち込む。どちらでも、同速の変更を高い確率で受理するには、CIの半幅より十分大きい許容幅が要り、それはownerが決める値で、測定からは導けない。

推奨はmargin 0で、これは測定の前に`records/registration.json`へ記録した。ownerの「遅くしない」をそのまま検査し、許容幅の根拠を必要としない。placebo marginは根拠のある許容幅に見えるが、上の2点のために、約束する意味を果たさない。ownerが決めるまでmargin 0を使う。placeboは判定ではなく解釈に使う。測定の後で分かった帰結も記す。下のとおりplacebo自身がmargin 0で3本「slower」になったので、margin 0のもとでは、挙動を変えずにrtlcのcodeを動かすだけの変更も受理されない。推奨は変えないが、この事実はownerの判断材料として示す。

### 測定

arm: before（3f-b9と同じ、b8の破棄変更だけを外した複製）、placebo、norm（beforeに`normalize`のin-place化だけを当てた複製）、a4（3f-b9の採用案）、p1n・p2n（下の試作）、b8（現行の作業木）。各armの`crates`を作業木外へ複製し、idealis-coreのrelease native binary（native-jsonl用）とwasm、計測harness（命令数と確保回数）、before・placeboだけはcriterionのbench実行ファイルをbuildし、成果物だけを残してbuild出力を消した（`records/artifacts.txt`）。順序は命令数 → in-process 27 process（7 arm）→ usable 129 block（7 arm、1,806 run）→ criterion 40 block（beforeとplacebo、160 run）で、1つずつ順に走らせた（`logs/measure-timeline.txt`）。

placeboの定義。beforeに(a4)の`bounded_drop.rs`を足し、`impl Drop`を外し、4つの入口を`#[used]`のstaticに置いた関数pointerで保持した。codeはbinaryに残るが呼ばれない。残っていることは大きさで確かめた（wasm +16,975 B、native binary +16,608 B、criterion bench +16,576 B。(a4)のwasmは+25,068 B）。確保回数はbeforeと一致し、命令数はsize 1・16・64で−0.47%・+0.09%・+0.20%で、実行する仕事が変わらないことと整合する。限界として、afterのinline gateのような呼出し箇所ごとのcode増加は再現しない。

in-process（判定2本。比はafter/beforeの幾何平均、括弧は[L, U]）:

| arm     | 16/wasm-inprocess       | 64/wasm-inprocess       | margin 0     | placebo margin |
| ------- | ----------------------- | ----------------------- | ------------ | -------------- |
| placebo | 0.9991 [0.9978, 1.0004] | 0.9998 [0.9986, 1.0010] | inconclusive | 閾値の元       |
| norm    | 0.9968 [0.9952, 0.9983] | 0.9900 [0.9884, 0.9916] | not-slower   | not-slower     |
| a4      | 1.0058 [1.0041, 1.0075] | 0.9990 [0.9972, 1.0008] | slower       | slower         |
| p1n     | 1.0085 [1.0068, 1.0101] | 1.0097 [1.0078, 1.0116] | slower       | slower         |
| p2n     | 1.0098 [1.0085, 1.0111] | 1.0107 [1.0089, 1.0126] | slower       | slower         |
| b8      | 1.0644 [1.0631, 1.0657] | 1.0745 [1.0726, 1.0764] | slower       | slower         |

placebo marginの閾値は1.0004と1.0010。破棄だけの費用（同じprocessでのnorm比、診断）は、a4 1.0090 [1.0080, 1.0101]・1.0091 [1.0081, 1.0102]、p1n 1.0117・1.0199、p2n 1.0131・1.0210、b8 1.0679・1.0854である。(a4)の合計は、破棄の約+0.9%をin-place化の−0.3%（16）・−1.0%（64）が打ち消した値で、16では打ち消し切れていない。in-place化はbeforeからあった確保を減らす別の改善で、破棄の費用が0である証拠にはならない。

usable（判定4本）:

| arm     | 16/wasm-wire            | 64/wasm-wire            | 16/native-jsonl         | 64/native-jsonl         | margin 0 | placebo margin |
| ------- | ----------------------- | ----------------------- | ----------------------- | ----------------------- | -------- | -------------- |
| placebo | 1.0044 [1.0007, 1.0081] | 0.9967 [0.9917, 1.0017] | 0.9997 [0.9945, 1.0049] | 0.9978 [0.9937, 1.0019] | slower   | 閾値の元       |
| norm    | 1.0108 [1.0068, 1.0149] | 1.0030 [0.9979, 1.0081] | 1.0003 [0.9956, 1.0051] | 0.9950 [0.9912, 0.9989] | slower   | inconclusive   |
| a4      | 1.0206 [1.0160, 1.0253] | 1.0078 [1.0023, 1.0133] | 1.0091 [1.0033, 1.0150] | 1.0083 [1.0028, 1.0138] | slower   | slower         |
| p1n     | 1.0172 [1.0136, 1.0208] | 1.0065 [1.0018, 1.0112] | 1.0136 [1.0084, 1.0188] | 1.0098 [1.0060, 1.0137] | slower   | slower         |
| p2n     | 1.0200 [1.0163, 1.0237] | 1.0085 [1.0038, 1.0131] | 1.0081 [1.0027, 1.0135] | 1.0080 [1.0038, 1.0122] | slower   | slower         |
| b8      | 1.0830 [1.0788, 1.0871] | 1.0597 [1.0548, 1.0647] | 1.0581 [1.0525, 1.0638] | 1.0541 [1.0496, 1.0587] | slower   | slower         |

usableの対照（両側で同じTypeScript）の比:

| arm     | 16/ts-prepared | 64/ts-prepared | 16/ts-wire | 64/ts-wire |
| ------- | -------------- | -------------- | ---------- | ---------- |
| placebo | 1.0057         | 1.0065*        | 1.0176*    | 1.0076*    |
| norm    | 1.0085*        | 1.0074*        | 1.0058*    | 1.0030     |
| a4      | 1.0114*        | 1.0105*        | 1.0198*    | 1.0105*    |
| p1n     | 1.0052         | 1.0023         | 1.0165*    | 1.0062*    |
| p2n     | 1.0082*        | 1.0074*        | 1.0213*    | 1.0078*    |
| b8      | 0.9854*        | 1.0072*        | 1.0042     | 1.0015     |

*は両側95%区間が1を含まないもの。24組中18組が1を外れ、ずれは−1.5%から+2.1%で、時間の4区間に分けても同じ向きに残る（例えばb8の16/ts-preparedは−2.5、−1.3、−1.3、−0.9%）。原因を切り分けるため、結果を見た後に足した診断として、beforeと同じbinaryを名前の長さが違うdirectory（`aa`、`beforx`）へ複製し、60 blockで同じ測定をした（`records/usable-aa.json`）。16組中1組（`aa`の16/native-jsonl、L 1.0007）だけが1を外れ、偶然の範囲である。ずれは実行環境ではなく、同じprocessに載るwasmとnativeのbinaryが変わったことから来る。usableは、変わっていないTypeScriptのpathでも、binaryの配置の違いで最大2%動く。したがってusableの判定は、2%程度までの差を変更の仕事に帰属できない。placeboが16/wasm-wireで「slower」になったのもこの範囲である。

criterion（placeboだけ、判定14本）:

| path                               | placebo/before          | margin 0     |
| ---------------------------------- | ----------------------- | ------------ |
| `membership/function-cube`         | 0.9953 [0.9929, 0.9978] | not-slower   |
| `membership/parallel-channels`     | 0.9955 [0.9933, 0.9978] | not-slower   |
| `search/counter-product/1`         | 1.0002 [0.9980, 1.0024] | inconclusive |
| `search/counter-product/8`         | 1.0011 [0.9990, 1.0032] | inconclusive |
| `search/function-cube/1`           | 1.0001 [0.9965, 1.0037] | inconclusive |
| `search/function-cube/8`           | 1.0058 [1.0009, 1.0108] | slower       |
| `search/linear-ring/1`             | 0.9960 [0.9922, 0.9999] | not-slower   |
| `search/linear-ring/8`             | 0.9959 [0.9928, 0.9991] | not-slower   |
| `search/parallel-channels/1`       | 0.9988 [0.9953, 1.0024] | inconclusive |
| `search/parallel-channels/8`       | 1.0174 [1.0124, 1.0225] | slower       |
| `values/except-function`           | 1.0003 [0.9996, 1.0010] | inconclusive |
| `values/except-record`             | 0.9987 [0.9977, 0.9997] | not-slower   |
| `values/intern-existing-functions` | 1.0030 [0.9998, 1.0062] | inconclusive |
| `values/intern-new-functions`      | 1.0018 [0.9988, 1.0047] | inconclusive |

placeboの結果。挙動を変えない変更は、in-processではどちらのsizeでも1と区別できない（2系列目でも0.9989 [0.9976, 1.0003]、1.0015 [0.9996, 1.0034]）。criterionでは5本が速く、2本（8 workerのsearch）が遅いと示され、配置だけで−0.5%から+1.7%動く。usableでは16/wasm-wireが+0.44%で「slower」、対照が3本ずれる。margin 0の判定は、placebo自身に対してslower（3本）である。3f-b9が`values/intern-*`の0.5〜2%を配置の揺れと推測した点は、この測定では+0.30%・+0.18%（どちらもinconclusive）で、配置の揺れはほかのpathに出た。

(a4)とb8を両方のmarginに当てた結果。新しい測定ではin-processとusableで、どちらのarmもmargin 0・placebo marginの両方でslowerである。(a4)はin-processの16/wasm-inprocessだけでslowerが決まり、このpathではplaceboがずれないので、usableの偏りに頼らない結論である。保存した3f-b9の記録だけ（登録したblock数に届かないので暫定）でも、margin 0で(a4)はusableの16/wasm-wire（L 1.0067）・64/native-jsonl（L 1.0027）・in-processの64（L 1.0005）がslower（criterionは2 blockしかなく、`values/except-function`だけL 1.0025でslower）、b8はusableとin-processの6本すべてがslower（Lは1.0257以上）だった（`records/saved-*-zero.txt`）。保存記録にはplaceboがないので、placebo marginは保存記録では計算できない。criterionは(a4)とb8について新しく測っていない。どちらもすでにslowerが決まっているので判定は変わらないが、criterionの値を報告するには、(before, placebo, after)の鏡像blockを40 block（各blockで6 run、1 armあたり約2時間）測る必要がある。

### 残差を消す試作

3f-b9の仮説は、(a4)の残差は`Value`のgateが子と孫を読み、その後drop glueが同じ子を読み直すことから来る、というものだった。これを崩す3案を試作した。どれも`normalize`のin-place化を含み（(a4)と同じ`value.rs`）、IRの3型は(a4)のまま、予算B=32と境界（`Value::List` 34段で0回・35段で1回、`Not` 33段で0回・34段で1回）も(a4)と同じである。3案とも凍結した全suite（rtlc `cargo test --all-targets` 207件、idealis-core 43件）と新しい2ファイル（10件・11件）を変えずに通った（`logs/tests-*.log`、`logs/draft-*.log`）。

- P1（drop内で子をその場で空にする）: `Drop for Value`のinline部分は自分が葉か（判別子と長さ）だけを読む。葉でなければ`#[inline(never)]`の`release_value`へ入り、子がすべて葉ならすぐ戻る。そうでなければ予算付きの後順走査で子を`&mut`で辿り、非葉の子をその場で空にしてから`clear()`で破棄する。glueに残るのは空の容器と葉だけで、子を読み直す深い走査は残らない。予算0では、flatでない子だけを`Value::Null`と入れ替えてworklistへ積む。
- P2（判別子と長さだけを読む安いgate）: inline部分は自分と直接の子の判別子と長さだけを読み（孫は読まない）、flatでなければP1と同じその場の走査へ入る。
- P3（gateの容器部分をout-of-lineにする）: P1・P2の結果を見た後に足した案で、(a4)のgateのうち葉の判定だけをinlineに残し、高さ2の判定と以降を`#[inline(never)]`の関数へ移す。`Value`のdrop glueは(a4)の375命令から96命令になる（beforeは88命令）。別の27 processの系列で、before・placebo・norm・a4と一緒に同じ計算で測った（`records/acceptance-p3-inprocess.json`）。

| 案  | in-process before比（16 / 64）                    | norm比の破棄費用（16 / 64） | 命令数 before比（1 / 16 / 64）                                     |
| --- | ------------------------------------------------- | --------------------------- | ------------------------------------------------------------------ |
| a4  | 1.0058 [1.0041, 1.0075] / 0.9990 [0.9972, 1.0008] | 1.0090 / 1.0091             | −1.02% / −0.58% / −0.61%                                           |
| P1  | 1.0085 [1.0068, 1.0101] / 1.0097 [1.0078, 1.0116] | 1.0117 / 1.0199             | −0.45% / +0.32% / +0.91%                                           |
| P2  | 1.0098 [1.0085, 1.0111] / 1.0107 [1.0089, 1.0126] | 1.0131 / 1.0210             | −0.48% / +0.02% / +0.13%                                           |
| P3  | 1.0072 [1.0063, 1.0082] / 1.0039 [1.0015, 1.0063] | 1.0109 / 1.0130             | −0.20% / −0.10% / +0.41%（同じ系列のa4は−0.60% / −0.71% / −0.59%） |

P3の系列では(a4)も1.0060 [1.0046, 1.0074]・1.0019 [1.0002, 1.0036]で、P3/(a4)は1.0012 [1.0001, 1.0024]・1.0019 [0.9991, 1.0048]だった。どの案もplaceboと区別できる。in-processの16で4案ともU_pがplaceboの閾値1.0004を超え、Lも1を超える。残差を消せた案はない。その場で空にする走査（P1・P2）は、gateで子と孫を読む(a4)より遅く、gateをout-of-lineにする案（P3）も遅い。確保回数は4案ともnormと同じ（size 64で66,052回）で、差は確保ではなく走査と呼出しの命令数から来る。ただしこれは測った3案についての結論で、ほかの技法で残差を消せないという証明ではない。

### 採用する設計と判定

破棄の設計は3f-b9の(a4)のまま変えない（予算付き再帰、`Value`の高さ2までのglue委任、`normalize`のin-place化）。測った有界の案のうち破棄の費用が最も小さいからで、残差を消せたからではない。

この設計は固定した受入れ計算を満たさない。in-processの16/wasm-inprocessがbefore比1.0058 [1.0041, 1.0075]（P3の系列で1.0060 [1.0046, 1.0074]）で、margin 0でもplacebo marginでもslowerである。実装段へ進んでも受入れはGreenにならない。どう進めるかはownerが決めることで、選択肢は次のとおりである。

- 測った費用を許容幅として明示的に認める。根拠は測定値で、wasmの破棄の費用がnorm比+0.9〜1.1%（in-process）、in-place化と合わせた合計がbefore比+0.6%（16）・−0.1〜+0.2%（64）である。許容幅を決めるのはownerで、測定から導いた値ではない。
- このpull requestでは`Value`の有界破棄を入れない。作業指示の「公開APIで作れるすべての値で有界stack」を外すことになる。
- 破棄の費用をほかの改善で打ち消す。ただしmargin 0はcriterionで配置だけが動くpathにも速さの証明を求めるので、打ち消しだけでは受理にならない。

### stackの上限

上限は予算B（= 32）だけで決まり、入力の深さによらないO(B)である。(a4)のsource上の最長経路は次のとおりで、`5B+8`ではない。

- 根4段: 呼出し元のdrop glue、`<Expression as Drop>::drop`、`release`、そのclosure。
- 予算の各段（b = B, B−1, …, 0のB+1段）に最大5段: `detach_expression → detach_bindings → detach_domain → dispose_expressions → dispose_expression`（`Domain::Set`を持つ量化子）。b ≥ 1の段の最後の`dispose_expression`がb−1で次の`detach_expression`を呼び、b = 0の段の最後がworklistへ積む。
- 底4段: `Vec::push → RawVecInner::grow_one → finish_grow →` allocatorの入口。allocator内部の段は数えない。

合計は4 + 5(B+1) + 4 = 5B+13段（B = 32で173段）である。空になったIR nodeのDropへの再入は、b = 1の段の`dispose_expression`から始まる「drop glue → Drop::drop → release → closure → detach_expression → detach_bindings」の6段で、4 + 5B + 6 = 5B+10段に収まる。`Value`だけの経路は根5段（`release_value`が加わる）と各段3段（`detach_value → dispose_value → descend_value`）で、5 + 3(B+1) + 4 = 3B+12段である。

byte数は見積りで、rustc 1.100.0と次のprofileに限る。各関数のframeはprologueのstack調整量から読み（`scripts/frames.py`、`scripts/wasm-frames.py`）、inline展開された関数は呼出し元のframeに含まれるので0とした。1段の値はsource上の段の組23通り（`scripts/stackbound.py`の`CYCLES`）の最大で、合計は根 + (B+1)·1段 + 底（push経路と再入の大きいほう）とした。

| profile                                                                                 | 1段の最大                | 根    | 底              | 合計の見積り |
| --------------------------------------------------------------------------------------- | ------------------------ | ----- | --------------- | ------------ |
| aarch64-apple-darwin release（LTO、codegen-units 1、opt-level 3、panic abort、harness） | 944 B（`Domain::Set`）   | 464 B | 848 B（再入）   | 約31.7 KiB   |
| aarch64-apple-darwin debug（`cargo test`のdeep_drop）                                   | 3,072 B（`Domain::Set`） | 704 B | 2,768 B（再入） | 約102.4 KiB  |
| wasm32-unknown-unknown release（shadow stack、engineのnative stackは別）                | 368 B                    | 160 B | 400 B（再入）   | 約12.4 KiB   |

allocator内部と呼出し元が既に使ったstackは含まない。inline属性はhintなので、別のcompilerや設定では段の合併が変わり得る。深さに依らない性質は予算の減算だけから来て、inlineには依存しない。凍結したDEEPの512 KiB threadに対しては、debugの見積りで約5倍の余裕がある。抽出の出力は`frames/`（`native-release-a4.txt`、`native-debug-a4.txt`、`wasm-release-a4.txt`、`stackbound-a4.json`。P1・P2の分も同じ場所）にある。

### テスト（実装の前に書く、凍結リストの外）

凍結68ファイルは変えない（開始時・終了時とも68/68一致）。テスト段で次の2ファイルを足し、段の終わりに凍結リストへ加えて70件とする。下書きは`drafts/drop_budget.rs`と`drafts/value_normalize_errors.rs`にある。

`crates/rtlc/tests/drop_budget.rs`（10件）。計数`#[global_allocator]`はdrop_allocations.rsと同じ形で、各caseを8 MiBのthreadで公開APIだけから構築・破棄し、破棄中の確保が期待回数であることと、生存block・byteが構築前に戻る（全解放）ことを要求する。3f-b9の5件（`Value::List` 34段で0回・35段で1回、`Not` 33段で0回・34段で1回、通常の値とprogram）にC2の5件を足した。

1. `shallow_quantifier_with_a_non_leaf_value_domain_drops_without_allocation`: レビューの`Forall { bounds: [Binding { domain: Value(Not(Literal)) }], body: Literal }`。確保0。
2. `every_expression_rule_over_a_shallow_non_leaf_child_drops_without_allocation`: `expression_cycle`の24種のwrapperそれぞれを浅い非葉の子`Not(Literal)`へ1回だけ当てた24 case。各case確保0。
3. `every_other_expression_owner_edge_over_a_shallow_non_leaf_child_drops_without_allocation`: 24種が通らない所有辺21本（二項の右、Ifの条件・帰結、Subseqのsequence・from、Atのkey、Updateのtarget、量化子のbody、`Domain::Range`のupper、`Domain::Set`、Chooseのbody、Foldのinitial・collection、Inのvalue、`Step::Bind`・`Step::Choose`のdomain、`Step::Frame`のarguments・captures、`Step::Guard`、`Expression::Data`の高さ3の値、MapSetの`Domain::Value`）。各case確保0。
4. `every_formula_owner_edge_over_a_shallow_non_leaf_child_drops_without_allocation`: `wrap_formula`の9種と、Frameのarguments・captures、非葉のAtom、Orの右、Forall・Existsのdomainの15 case。各case確保0。
5. `every_value_container_over_a_non_flat_child_drops_without_allocation`: `wrap_value`の5種（List、Set、Recordの値、Mapのkey、Mapの値）を高さ2の子へ当てた5 case。各case確保0。

`crates/rtlc/tests/value_normalize_errors.rs`（11件）。期待値は変更前（HEAD）の`Value::normalize`から取った。各要素を入力順に処理し、Recordは値の正規化の後に名前の重複を、Mapはkeyの正規化 → keyの符号化 → 値の正規化の後にkeyの重複を判定する。`CoreError`の`code`と`message`を組で比べる。

- Record: `[("a", Null), ("a", Int("01"))]`は`("value", "Noncanonical integer")`。`[("a", Null), ("a", Null), ("b", Int("01"))]`は`("value", "Duplicate record field")`。`[("b", Int("01")), ("a", Bytes("F"))]`は入力順で先の`Noncanonical integer`。
- Map: `[(ModelValue("1x"), Int("01"))]`はkeyの`Invalid model value name`。`[(Int("1"), Null), (Int("01"), Null)]`はkeyの`Noncanonical integer`。`[(Int("1"), Null), (Int("1"), Bytes("F"))]`は値の`Invalid lowercase hex bytes`。`[(Int("1"), Null), (Int("1"), Null), (Int("2"), Int("01"))]`は`Duplicate map key`。
- List: `[Int("1"), Bytes("F"), Int("01")]`は先の`Invalid lowercase hex bytes`。`[List([ModelValue("")]), Int("01")]`は入れ子の先の`Invalid model value name`。
- Set: `[Int("1"), ModelValue(""), Int("01")]`は先の`Invalid model value name`。`[Int("2"), Int("2"), Bytes("F")]`は重複を誤りにせず、後の`Invalid lowercase hex bytes`。

TypeScriptの`portable-value.ts`は全要素を先に正規化してから重複を見るので、Recordの2件目のような入力では報告が違う。これは既存の差でこの変更の範囲外であり、このテストはRustの現行の契約を固定するもので、両言語の一致は主張しない。

赤と感度。現行の作業木（b8）の複製に当てると、drop_budgetの10件中8件が`allocations during the drop for case …`のassertion失敗（left 1、right 0）で落ち、abortはしない。境界の上側2件（35段・34段で1回）はb8でも通る。beforeの複製では逆に上側2件だけが落ちる（left 0、right 1）。value_normalize_errorsはb8でもbeforeでも通る。挙動を保つための防護で、まだ壊れていないからである。そこで、作業木外の複製に誤った実装を1つずつ入れ、(a4)とP2の両方を土台にして感度を確かめた（`scripts/mutants.py`、`records/mutants-a4.json`、`records/mutants-p2n.json`）。17の誤りはすべて、狙ったテストだけで落ちた。caseは各テストのloopで最初に落ちたものである。

| 誤った実装                                   | 落ちるテスト（assertion）                                                                      |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `Domain::Value`へ予算0を渡す（レビューの例） | 1（case 0）、2（case 16）、3（case 14）、4（case 13）。どれもleft 1、right 0                   |
| `Domain::Range`のupperへ予算0                | 3（case 8）                                                                                    |
| `Step::Frame`のargumentsへ予算0              | 3（case 16）                                                                                   |
| 二項演算の右へ予算0                          | 3（case 0）                                                                                    |
| formulaのFrameのcapturesへ予算0              | 4（case 10）                                                                                   |
| `Expression::Data`へ予算0                    | 3（case 19）、通常の値とprogram                                                                |
| `Value`のList要素へ予算0                     | 3（case 19）、5（case 0）、通常の値、`Value::List` 34段                                        |
| `Value`のMapの値へ予算0                      | 5（case 4）、通常の値                                                                          |
| `DEPTH_BUDGET`を33 / 31                      | 35段・34段の上側2件（left 0、right 1）/ 34段・33段の下側2件（left 1、right 0）                 |
| Recordで名前の重複を値より先に見る           | `record_reports_a_field_value_error_before_a_duplicate_name`                                   |
| Recordで全値を先に正規化する                 | `record_reports_a_duplicate_name_before_a_later_field_value_error`                             |
| Recordを名前順に並べてから正規化する         | `record_reports_the_first_failing_field_in_input_order`                                        |
| Mapで値をkeyより先に正規化する               | `map_reports_a_key_error_before_its_value_error`                                               |
| Mapで重複を値より先に見る                    | `map_reports_a_value_error_before_a_duplicate_key`                                             |
| Listを後ろから正規化する                     | `list_reports_its_first_failing_element`、`list_reports_an_error_nested_in_an_earlier_element` |
| Setを後ろから正規化する                      | `set_reports_its_first_failing_element`                                                        |

実装段は凍結した70ファイルを変えない。既存の89件、drop_allocations 8件、新しい21件を1件ずつ別processで実行し、b8で記録したsuiteをGreenに保ってから、上の受入れ計算を行う。

### 残すもの

- (a4)は固定した受入れ計算でslowerであり、ownerの判断なしには受入れがGreenにならない。
- usableは、変わっていないTypeScriptのpathでもbinaryの配置の違いで最大2%動く。この大きさまでの差は、usableだけでは変更の仕事に帰属できない。
- margin 0では、挙動を変えない変更（placebo）もslowerになる。どちらのmarginでも、同速の変更を高い確率で受理する許容幅は測定から導けない。
- `Value::normalize`以外の深い値の再帰（Clone・PartialEq・Hash・Debug・Serialize、`Arena::data`）はb8の残課題のままである。

## step 3f-b11 design: ownerの許容幅を事前登録した受入れ計算へ

改訂: 2026-11-18 初版（前提レビュー前の草案）。3f-b10からの変更と理由は次のとおり。

- ownerの決定（2026-11-18、選択肢A）を記録した。3f-b10の(a4)はmargin 0でもplacebo marginでもslowerで、進め方をownerに委ねていたため。
- margin 0とplacebo marginを削除し、判定ごとに1つの固定閾値「決定で認めた値 ×（1 + Δ）」へ置き換えた。Δは3f-b10が測定の前に登録したin-processの分解能0.4%である。margin 0は決定で認めた(a4)を必ず落とし、placebo marginは閾値がplacebo系列の雑音とともに動く（C1の一部を持ち込む）ため。
- 判定をin-processの4件（size 16・64のbefore比と、norm比の破棄費用）に絞り、usable 8本とcriterion 14本を記録だけにした。配置だけでこの2つのfamilyが動く大きさ（usable最大2%、criterion −0.5〜+1.7%）が認めた大きさを超え、許容幅を広げずには判定できないため。
- 判定pathが観測に1行もないとき黙って飛ばしていた分岐を拒否に変え、保存記録用の暫定mode（`--provisional`）を外した。3f-b10の`accept.py`は集計の部分集合のためにこの分岐を持っていたが、判定pathの欠落で受理しないため。
- 記録familyの拒否はその場で報告し、判定を変えないことにした。判定は最初に測るin-processだけで決めて封印し、後のrunの失敗が再測定の理由にならないようにするため。
- 実装の測定は、上流main（d147550c、#61）をbranchへ統合した後に、統合した木からすべてのarmを作って行うことにした。統合がすべてのarmのwasmとnative binaryを変えるため。

本番ソースとテストは変えていない。計算・自己検査・確率の解析・dry runは`<scratch>/step3f-b11-design/`（以下、記録の場所はここからの相対path）に置いた。3f-b10の記録は`<scratch>/step3f-b10-design/`にあり、`b10:`を付けて示す。

### ownerの決定（2026-11-18）

ownerは選択肢Aを選んだ。3f-b10で測った(a4)の費用を、明示的な許容幅として認める。ownerは以前、許容幅が何に基づくのかを問うた。答えはこの決定である。3f-b10の測定値についてのownerの判断であり、統計からの導出でも、run間の散らばりでもない。

| 認めた量                                           | 決定の値    | 元の測定値（比 [L, U]）                              | 記録                                                                                           |
| -------------------------------------------------- | ----------- | ---------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| in-process wasm、size 16、before比（合計）         | +0.6%       | 1.0058 [1.0041, 1.0075]、1.0060 [1.0046, 1.0074]     | `b10:records/acceptance-summary.json`（a4、zero）、`b10:records/acceptance-p3-inprocess.json`  |
| in-process wasm、size 64、before比（合計）         | −0.1〜+0.2% | 0.9990 [0.9972, 1.0008]、1.0019 [1.0002, 1.0036]     | 同上                                                                                           |
| in-process wasm、size 16・64、norm比（破棄の費用） | +0.9〜1.1%  | 1.0090、1.0091（1系列目）、1.0096、1.0111（2系列目） | `b10:records/acceptance-summary.json`の`dropCostOverNorm`、`b10:records/observations-p3.jsonl` |

1系列目は3f-b10の本測定（27 process、7 arm）、2系列目はP3を測った別の27 processである。2系列目のa4/norm（1.0096 [1.0085, 1.0108]、1.0111 [1.0090, 1.0131]）は3f-b10の本文に出していない値で、同じ記録から同じ計算で求めた（`records/probability.json`の`measured`）。usableとcriterionについて、決定は大きさを述べていない。usableの(a4)の値は配置の偏りを含み、criterionは(a4)を測っていないからである。

### 受入れ計算（実装の測定の前に固定）

計算は`scripts/accept.py`（`REGISTRATION`定数）と`scripts/tdist.py`（3f-b10とbyte単位で同一）にあり、`records/registration.json`と`records/registration.sha256`へ記録した（2026-11-17T22:24Z、UTC）。以後この計算は変えない。

統計は3f-b10のまま変えない。鏡像block（in-processはprocess）ごとの対数比d_j、片側97.5%の上側限界U_p = exp(d̄ + t(0.975, n−1)·s/√n)、判定すべての交わりで受理する規則（intersection-union、Berger 1982）、拒否の規則、再測定で判定を変えないことである。新しいcodeは保存した(a4)の3f-b10の限界をbit単位で再現する（自己検査）。結果の種類も同じで、判定すべてでU ≤ 閾値ならaccept、どれかでL > 閾値ならslower、それ以外はinconclusiveで、inconclusiveはGreenにしない。

変えるのは閾値だけである。閾値は判定ごとに1つの固定数で、「決定で認めた値 ×（1 + Δ）を小数4桁へ切り捨てた値」とする。Δ = 0.4%は3f-b10が自身の測定の前に登録したin-processの分解能（n = 27がこの差を見分けるように決めた値で、3f-b9の(a4)の最小残差+0.37%から来る）である。

| family・判定                                                      | 扱い | 閾値                  | 決定のどの値か                |
| ----------------------------------------------------------------- | ---- | --------------------- | ----------------------------- |
| in-process `16/wasm-inprocess`、impl/before                       | 判定 | 1.0100                | 合計+0.6% × 1.004             |
| in-process `64/wasm-inprocess`、impl/before                       | 判定 | 1.0060                | 合計の上端+0.2% × 1.004       |
| in-process 16・64、impl/norm（破棄の費用）                        | 判定 | 1.0150（familyで1つ） | 破棄の費用の上端+1.1% × 1.004 |
| in-process placebo/before、norm/before                            | 記録 | なし                  | 配置の対照、`normalize`の別掲 |
| usable 8本（impl/before、impl/norm、placebo/before、norm/before） | 記録 | なし                  | 決定は大きさを述べていない    |
| criterion 14本（impl/before、placebo/before）                     | 記録 | なし                  | 決定は大きさを述べていない    |

閾値が認めた値そのものではなく1 + Δを掛けたものである理由。U ≤ Tは「真の比がT以下であることを97.5%の確からしさで示した」という意味である。認めた値は(a4)の測定値そのものなので、T = 認めた値とすると(a4)と同じ実装はTのちょうど上にいて、ほとんど通らない（保存系列から求めると受理は0.057と0.001）。T = 3f-b10で測った(a4)自身のUとしても各判定が約半分しか通らず、4件の交わりでは0.178と0.171である。決定が認めた実装を通すには、測定の分解能だけTを上へ置く必要がある。その幅には、測定の前に登録してあった分解能Δをそのまま使い、この測定や次の測定のどちらの側の散らばりからも取らない。閾値は判定ごとに一度述べた固定数で、何回測っても同じであり、C1は戻らない。

それ以上ではない理由。Δは配置の吸収ではない。in-processのplaceboは同じprocessで0.9991・0.9998、2系列目で0.9989・1.0015で、配置だけの動きは−0.11〜+0.15%とΔより小さい。usableの2%やcriterionの1.7%に合わせて広げてはいない。そして、どの判定でも認めた費用より1%遅い実装は確率1で拒否される（下の確率）。認めた費用からTまでの0.4%は、この測定が(a4)と見分けられない灰色の帯で、ここに入る実装は通り得る。これは決定の帰結として明示する。

64の合計に上端+0.2%を使うのは、決定の「−0.1〜+0.2%」が2系列の測定値の幅で、その上端までが認めた値だからである。破棄の費用を合計とは別に判定するのは、合計だけでは`normalize`の改善が破棄の費用の増加を隠せるのに対し、ownerは破棄の費用を別に認めたからである。`normalize`のin-place化そのもの（norm/before）は記録して別掲する。

#### 配置だけで値が動くfamily

- in-process（判定）。配置だけの動きは上のとおりΔより小さい。placeboを同じprocessに置いて記録し、統合した木でも配置の大きさを示す。placeboが±0.4%を外れても判定は変えず（再測定しない）、配置が分解能を超えたことを記録に残す。
- usable（記録だけ）。認めた大きさの退行を、usableは見分けられない。両側で同じTypeScriptを測る対照が、binaryの違いだけで−1.5〜+2.1%動いた（3f-b10）。(a4)自身の対照も+1.05〜+1.98%で、16/wasm-wireはusableで1.0206なのに、同じwasmの呼出しをin-processで測ると1.0058である。差の約1.5%は変更の仕事ではない。同じblockの対照でもこれは除けない。placeboにはplacebo自身の配置があり（16/wasm-wire +0.44%に対し16/ts-wire +1.76%）、実装の配置はそれとも違う。TypeScriptの対照で割る補正も合わない（(a4)の16/wasm-wire ÷ 16/ts-wireは1.0008でin-processの1.0058と違い、b8は1.0785で1.0644と違う）。beforeと同じbinaryを別のdirectoryに置いた対照（`aa`・`beforx`）は環境が安定していることを示すだけで、binaryごとの配置は消さない。in-processの閾値でusableを判定すると、(a4)と同じ実装が通る確率は0.000で、95%で通すには1.0182〜1.0296が要る（`records/probability.json`）。これは配置を吸収するための拡大なので行わず、記録だけにする。
- criterion（記録だけ）。placeboだけで−0.5〜+1.7%動き（`search/parallel-channels/8`は1.0174 [1.0124, 1.0225]で、閾値1.0150ではplaceboだけでinconclusive、1.0100ではslower）、(a4)は測っていない。`values/intern-*`のように変更した4型を破棄しないpathもある。決定はcriterionの大きさを述べておらず、placeboにも固有の配置があるので、認めた大きさの退行を見分けられない。記録だけにする。

これで判定されずに残るもの。native codeの実時間（usableのnative-jsonlとcriterion全体）と、新しいprocessでのwire経路である。nativeにだけ現れる認めた大きさの退行（aarch64でのgateの生成code、inlineの判断、`search/*/8`のrayon worker上の破棄）は判定しない。部分的に覆うものは次の3つで、どれも実時間の判定ではない。同じRustの破棄と`normalize`のsourceがwasmでin-processに判定される（余分な走査や確保のようなalgorithmの退行はそこに現れる）。各armのnativeの命令数と確保回数を診断として記録する（3f-b10の(a4)/normの命令数はsize 1・16・64で+0.31%・+1.01%・+1.30%）。drop_budgetの10件がnative（debug）で破棄中の確保回数を要求する。

### 通る確率と拒否される確率（保存した3f-b10の系列から）

`scripts/probability.py`で、保存した2系列それぞれから27 processを復元抽出で20,000回引き（1つのprocessの全armを一緒に引くので判定間の相関を保つ）、(a4)を実装として登録した計算を当てた。a4の対数比に一様な遅さを足した行は、統合や系列の違いで比が動いた場合の感度でもある。

| (a4)に足した遅さ | 1系列目 accept | 1系列目 slower | 2系列目 accept | 2系列目 slower |
| ---------------- | -------------- | -------------- | -------------- | -------------- |
| −0.2%            | 1.000          | 0.000          | 1.000          | 0.000          |
| 0（(a4)と同一）  | 0.997          | 0.000          | 0.970          | 0.000          |
| +0.1%            | 0.955          | 0.000          | 0.802          | 0.000          |
| +0.2%            | 0.728          | 0.000          | 0.375          | 0.000          |
| +0.3%            | 0.320          | 0.000          | 0.043          | 0.004          |
| +0.4%            | 0.056          | 0.008          | 0.000          | 0.068          |
| +0.5%            | 0.001          | 0.129          | 0.000          | 0.496          |
| +0.6%            | 0.000          | 0.615          | 0.000          | 0.947          |
| +0.8%            | 0.000          | 1.000          | 0.000          | 1.000          |
| +1.0%            | 0.000          | 1.000          | 0.000          | 1.000          |

- (a4)と同じ実装は0.997と0.970で受理され、slowerにはならない。2系列目で最も弱いのは64のnorm比（a4/norm 1.0111、標準偏差0.0052）で、正規分布を仮定した計算でも0.967である。
- 認めた費用より1%遅い実装（どれか1つの判定を認めた値 × 1.01へ置き直した系列）は、4つの判定のどれでも、両系列でaccept 0.000、slower 1.000である。(a4)より認めた破棄の費用だけ遅い実装（(a4) × 1.011）もslower 1.000である。
- 受理が半分を切るのは(a4)より約0.25%（1系列目）・約0.17%（2系列目）遅いときで、0.4%遅ければほぼ通らない。

この確率は、統合した木での実装の比が保存したどちらかの系列の(a4)に等しいことを前提にする。2系列の(a4)は64のbefore比で0.29%、norm比で0.20%違い、系列内の標準誤差（約0.05〜0.10%）より大きい。系列の違いや統合でこの大きさだけ遅く動けば、受理は表の+0.1〜+0.3%の行まで下がる。これは決定の帰結で、測って認めた値を超える遅さを吸収するために閾値を広げはしない。その場合の判定はinconclusiveかslowerで、再測定せずownerへ返す。

### 実装の測定

前提: 上流main（d147550c）をbranchへ統合した後で、実装段とテスト段（凍結71ファイル、既存89件、drop_allocations 8件、新しい21件を1件ずつ別processでGreen）を終えていること。

arm（すべて統合した木から作り、baseを混ぜない）:

- impl: 統合した作業木の`crates`の複製（実装段の後、そのまま）。
- before: implから破棄の変更（`bounded_drop.rs`、その`mod`行、`impl Drop`）と`normalize`のin-place化を外した複製。
- norm: beforeに実装の`normalize`のin-place化だけを当てた複製。
- placebo: beforeに実装の`bounded_drop.rs`を足し、`impl Drop`を外して入口を`#[used]`のstaticに置いた複製。

各複製とimplの差分を記録し、差が上の範囲だけであることを確かめる。wasm、native binary、計測harness、criterionのbench（before・placebo・impl）のsha256を`records/artifacts.txt`の形で残す。TypeScriptの`dist`は統合した作業木から一度だけbuildし、全armで共有する。

| family           | arm                         | 単位と数                          | run数 | 扱い                            |
| ---------------- | --------------------------- | --------------------------------- | ----- | ------------------------------- |
| 命令数・確保回数 | before、placebo、norm、impl | 3 round × size 1・16・64          | 36    | 診断                            |
| in-process       | before、placebo、norm、impl | 27 process（各armが600回×2 size） | 27    | 判定（4件）                     |
| usable           | before、placebo、norm、impl | 129鏡像block（各armが2回）        | 1,032 | 記録                            |
| criterion        | before、placebo、impl       | 40鏡像block（各armが2回）         | 240   | 記録（3f-b10が必要と述べたrun） |

block数は3f-b10の登録値（27・129・40）を使う。判定が4件になったので、pathあたりの第2種の誤りは1 − 0.8^(1/4) = 5.4%でよく、3f-b10で実際に測った最大の標準偏差0.0052（2系列目の64のa4/norm）で0.4%を見分けるには24 processで足りる。`records/pilot.json`から計算し直さないのは、測った0.0052がpilotの80%上側限界0.0046を超えており、pilotでは少なく見積もるからである。27は両方を上回り、保存した系列を登録したnのままdry runに使える。usableとcriterionは3f-b10の記録と同じ精度で比べられるよう同じ数にする。criterionの鏡像blockは3 armなので、usableと同じく前半の順序をblockごとに回し、後半を逆順にする（3f-b10の`criterion-blocks.sh`は2 arm用だった）。

順序はbuild → 命令数・確保回数 → in-process → 判定 → usable → criterionで、1つずつ順に走らせる。判定はin-processの観測がそろった時点で計算し、その時刻とsha256を書いてからusableを始める（封印）。後のrunの結果や失敗は判定を変えない。測定scriptは3f-b10のもの（`b10:scripts/`）をarmと出力先だけ変えて使い、最初のrunの前にsha256を`records/registration.sha256`へ追記する。

時間の見積り（3f-b10の実績`b10:logs/measure-timeline.txt`、`b10:records/artifacts.txt`から）:

| 段               | 3f-b10の実績                                              | 見積り                                    |
| ---------------- | --------------------------------------------------------- | ----------------------------------------- |
| 複製とbuild      | wasm・nativeで1 armあたり46〜47秒、benchは24秒            | 約10分                                    |
| 命令数・確保回数 | 6 armで56秒                                               | 約40秒                                    |
| in-process       | 27 process × 7 armで555秒（1 arm・1 processあたり2.94秒） | 約5分20秒                                 |
| usable           | 1,806 runで628秒（1 runあたり0.35秒）                     | 約6分                                     |
| criterion        | 160 runで4,612秒（1 runあたり28.8秒）                     | 約1時間55分                               |
| 合計             |                                                           | 約2時間20分（判定は開始から約16分で封印） |

### 統合が3f-b10の数値に与える影響

上流のd147550cが変えるのは、idealis-coreの`lib.rs`（`Request::CompileTla`・`CheckTla`への`modules`と`compile_tla_bundle`）、rtlaのparserとAST、rtla-compilerの`link.rs`・`rewrite.rs`・`lib.rs`、TypeScriptの`contracts.ts`・`tla-source.ts`、`evals/criterion-safety/run.ts`・`verify.ts`、justfileと文書である。rtlcのsource・`Cargo.toml`・`Cargo.lock`、`evals/usable`、`check-finite`の経路は変えない。

- 閾値。ownerが統合前のin-processの値について決めた数で、計算し直さない。統合は決定を無効にしない。ただし変わったidealis-core・rtla・rtla-compilerのcodeはすべてのarmのwasmとnative binaryに入り、測る要求（`check-finite`）はその分岐を通らないが、codeの配置は全armで変わる。3f-b10のin-processとusableの比は統合前の測定であり、破棄の仕事（rtlcのcode）は変わらないが、配置の成分は動き得る。通る確率はこれが動かないことを前提にした値で、動いた場合の感度は上の表の+0.1〜+0.3%の行にある。したがって実装の測定は統合した木から全armを作り、3f-b10の成果物とは比べない。
- criterion。3f-b10はrtlcの`safety` bench実行ファイルを直接走らせ（`b10:scripts/criterion-blocks.sh`）、`evals/criterion-safety/run.ts`を使っていない。上流の`run.ts`・`verify.ts`の変更はrtla-compiler targetだけのもので（estimateが12から17、`module-inputs.json`の保存、`benchmarkSuite`）、rtlc targetは14本のままである。記録だけにする判断に使ったcriterionのplaceboの値は影響を受けず、統合後のrtlcのbenchも変わらないrtlcのsourceからbuildされる。justfileの`compiler-criterion`のplan名と`module-eval`は使わない。
- dry run。統合前に保存した値の再計算なので影響を受けず、どちらにしても受入れの証拠ではない。
- 番号と凍結。上流が0047を使うので、統合でこの記録は0048へ移る。`<scratch>`の記録は番号に依らない。凍結ファイルは68件のままで、統合でbyteが変わった1件（`evals/fixtures/tlc-operator-sharing/analysis-boundary-manifest.json`）のdigestだけを取り直す。71件になるのはテスト段の後である。

### 自己検査

`scripts/accept_selftest.py`は34/34で通った（`records/accept-selftest.txt`）。

- 3f-b10の8件を閾値1の登録で新しいcodeに当て、同じ結果になった（全組1%遅い系列とレビューの反例は非受理、等速の40系列でslowerは3以下・inconclusiveは35以上、2%速い系列は受理、runの欠落・不足したblock・0の値は拒否）。保存した(a4)の3f-b10の限界もbit単位で再現した。統計は変わっていない。
- 閾値は登録した規則（認めた値 × 1.004を4桁へ切り捨て）と一致する。
- (a4)と同じ実装: 保存した2系列はどちらもaccept。全判定を認めた値に置いた合成系列は40 seed中39でaccept。beforeと等しい実装は40/40、2%速い実装もaccept。
- 認めた費用より全組で1%遅い実装: 雑音なしでslower、雑音ありで40/40 slower、保存した2系列の(a4)を1.01倍した系列もslower。雑音を0.32%・1%・3%に増やしても60回中acceptは0で、雑音が許容に変わらない。
- C1の反例（beforeが[100×9, 102]の型、全組でa = 1.01·b）はslowerで拒否される。ただし拒否するのは64の合計（認めた+0.2%）である。size 16だけなら+1.0%はちょうど閾値1.0100の上にあり（認めた+0.6%と分解能0.4%）、norm比の+1.0%は認めた破棄の費用1.1%の内側である。C1が求めた性質、つまりbeforeの散らばりが遅い実装を通さないことは、反例がどこに落ちるかとは別に確かめた。同じ型で1つの判定だけを認めた費用より1%遅くすると4件とも単独でslowerになり、beforeの外れ値を102・120・200と広げても閾値も判定も変わらない。
- 64の閾値ちょうどの実装は200 seed中7でaccept（水準2.5%と整合）。
- 拒否: runの欠落、26 block、値が0・負・NaN・無限大、blockの釣り合いの崩れ（1 armのrunが1つ多い）、判定pathの欠落（新しい規則）、baselineのarm（norm）の欠落。記録familyの拒否はその場で報告され、判定を変えない。

負の対照として、登録したfileを変えずにメモリ上で閾値だけを替えて同じ自己検査を走らせた。認めた値 × 1.02にすると9件が落ち（C1の反例がaccept、雑音ありで1%遅い実装が60回中30回accept、閾値ちょうどの実装が200/200 accept）、認めた値そのものにすると5件が落ちた（保存した(a4)の2系列と合成系列が非受理）。判定pathの欠落を飛ばす3f-b10の分岐に戻すと、その1件が落ちた。

### dry run（受入れの証拠ではない）

保存した3f-b10の系列（統合前）を登録した計算で再計算した（`scripts/dryrun.py`、`records/dryrun.txt`・`records/dryrun.json`）。統合した木での測定はこれと違い得るので、これは受入れの証拠ではない。

| arm     | 系列 | 16 before比             | 64 before比             | 16 norm比               | 64 norm比               | 判定         |
| ------- | ---- | ----------------------- | ----------------------- | ----------------------- | ----------------------- | ------------ |
| a4      | 1    | 1.0058 [1.0041, 1.0075] | 0.9990 [0.9972, 1.0008] | 1.0090 [1.0080, 1.0101] | 1.0091 [1.0081, 1.0102] | accept       |
| a4      | 2    | 1.0060 [1.0046, 1.0074] | 1.0019 [1.0002, 1.0036] | 1.0096 [1.0085, 1.0108] | 1.0111 [1.0090, 1.0131] | accept       |
| p1n     | 1    | 1.0085 [1.0068, 1.0101] | 1.0097 [1.0078, 1.0116] | 1.0117 [1.0101, 1.0133] | 1.0199 [1.0183, 1.0216] | slower       |
| p2n     | 1    | 1.0098 [1.0085, 1.0111] | 1.0107 [1.0089, 1.0126] | 1.0131 [1.0114, 1.0147] | 1.0210 [1.0194, 1.0226] | slower       |
| p3n     | 2    | 1.0072 [1.0063, 1.0082] | 1.0039 [1.0015, 1.0063] | 1.0109 [1.0096, 1.0122] | 1.0130 [1.0104, 1.0156] | inconclusive |
| b8      | 1    | 1.0644 [1.0631, 1.0657] | 1.0745 [1.0726, 1.0764] | 1.0679 [1.0660, 1.0697] | 1.0854 [1.0833, 1.0875] | slower       |
| placebo | 1    | 0.9991 [0.9978, 1.0004] | 0.9998 [0.9986, 1.0010] | 1.0023 [1.0012, 1.0035] | 1.0099 [1.0085, 1.0113] | accept       |
| placebo | 2    | 0.9989 [0.9976, 1.0003] | 1.0015 [0.9996, 1.0034] | 1.0025 [1.0017, 1.0034] | 1.0106 [1.0085, 1.0127] | accept       |

閾値は左から1.0100・1.0060・1.0150・1.0150。p1nとp2nは64で認めた費用を明らかに超え、p3nは64でUが閾値をわずかに超えてinconclusiveになる。placeboは3f-b10のmargin 0では（usableとcriterionのpathで）slowerだったが、ここでは受理される。normを実装として当てるとnorm比は自分自身との比（1.0000）になり意味を持たないので、表から外した（判定はaccept）。記録familyは3f-b10と同じ値で（例えば(a4)のusable 16/wasm-wireは1.0206 [1.0160, 1.0253]）、(a4)のcriterionは測っていないので「欠落」として拒否の形で報告される。

### 3f-b10から変えないものと、決定が変えるもの

変えないもの:

- 破棄の設計は(a4)のまま（予算付き再帰、予算B = 32、`Value`の高さ2までのglue委任）。境界も同じ（`Value::List` 34段で0回・35段で1回、`Not` 33段で0回・34段で1回）。
- `normalize`のin-place化は破棄の費用と分けて報告する（norm arm、norm/beforeを記録、破棄の費用はimpl/norm）。
- テスト段で足す21件（`drop_budget.rs` 10件、`value_normalize_errors.rs` 11件）と、その赤・感度の記録。
- 凍結68ファイルは、テスト段の後で71ファイルになる。統合でbyteが変わる凍結ファイルはdigestを取り直すが、数と範囲は変わらない。
- stackの上限（O(B)、source上の最長経路5B+13段）とbyte数の見積り。見積りは統合前の試作binaryについての、rustc 1.100.0と記したprofileに限った値のままで、深さに依らない性質は予算の減算だけから来るので統合に依らない。
- 残すもの（`Value::normalize`以外の深い値の再帰、usableが配置で最大2%動くこと）。

決定が変えるもの:

- 3f-b10の判定path 20本（criterion 14、usable 4、in-process 2）を、in-processの4件の判定に替えた。usableとcriterionは判定から記録へ移った。決定が大きさを明示し、この2つのfamilyはその大きさを見分けられないからである。
- 3f-b10の「(a4)はownerの判断なしには受入れがGreenにならない」と「許容幅は測定から導けない」は、この決定で閉じた。許容幅はownerが測定値について決めたもので、受入れはこの登録のもとでGreenになり得る。
- 実装段の終わりに行う「上の受入れ計算」は、この節で登録した計算を指す。

### 残すもの

- usableとcriterionは記録だけで、native codeの実時間と新しいprocessでのwire経路は、認めた大きさでは判定しない。
- 認めた費用からその0.4%上までは灰色の帯で、(a4)より約0.2〜0.3%遅い実装も通り得る。0.5%以上遅ければほぼ通らない。
- 統合や系列の違いで比が0.2%以上動けば、(a4)と同じ実装もinconclusiveかslowerになり得る。その場合は再測定せずownerへ返す。
- `Value::normalize`以外の深い値の再帰（Clone・PartialEq・Hash・Debug・Serialize、`Arena::data`）はb8の残課題のままである。

## step 3f-b12 design: 受入れ計算の入力検査と、ownerが承認した許容幅

改訂: 2026-11-18 初版（前提レビュー前の草案）。3f-b11の前提レビューはNO-GO（C1・C2）だった。3f-b11からの変更と理由は次のとおり。

- 計算に3つの入力検査を足した（C2）。in-processでは各(path, block, arm)の値がちょうど1つ、usableとcriterionではちょうど2つであること。比べる各armのblock idが計画した0..n−1そのもの（in-processは27 process）であること。1つのfamilyの観測がすべて同じ1つのrun idを持つこと。3f-b11の計算は各blockの件数がそろっていることしか見ず、2系列を連結した54 processを27 blockへ平均して、inconclusiveをacceptに変えたため。
- converterを作り直した（C2）。計画したファイル名だけを読み、ほかのファイルがあれば拒否する。process idの重複、ファイル名と違うprocess id、別のrun id、登録と違うarm、登録した600と違う標本数を拒否する。3f-b10の`convert_new.py`は`proc-*.json`をすべて読み、block idを内容から取り、標本数を入力自身の`rounds`と比べていたため。
- 測定のrunに一意のrun idを付け、封印を測定scriptの段にした（C2）。runは「開いている」「封印済み」「報告済み」の3状態を持ち、封印済みのrunはin-processの観測を受け付けない。3f-b11の封印は手順の文章だけで、判定のあるrunへ観測を足すことを止める仕組みがなかったため。
- in-processとusableのharnessは出力にrun idを書き、process中の中央値を標準出力へ出さなくした。観測をrunに結び付け、判定に使う値を封印の前にログで見せないため。
- 3f-b11の閾値1.0100・1.0060・1.0150を、分解能Δからの導出としては撤回した（C1）。決定が認めていない0.4ポイントの上乗せだったため。2つの案を登録した。閾値を認めた値そのものにするExactと、3f-b11と同じ数をownerの承認だけを根拠に使うWidthである。ownerは2026-11-18にWidthを選び、上乗せを明示的に承認した（2つ目の決定）。Exactは選ばれなかった案として、確率とともに残す。
- 「確率1で拒否」を「20,000回の抽出で誤受理0回」へ、「97.5%の確からしさ」を頻度論の言い方へ改めた（レビューの限定）。
- 判定しないものにExpression・Branch・TemporalFormulaの破棄経路を加えた（レビューの質問3）。3f-b11は、同じRustの破棄のsourceがin-processで判定されると書いたが、in-processのharnessは`checkFiniteHistories`だけを走らせ、有限modelの`Value`しか通らない。usableとin-processの差約1.5%を変更の仕事でないと因果で分けたわけではない、とも限定した。
- 自己検査を34件から116件にし、4つの負の対照を足した。

本番ソースとテストは変えていない。計算・converter・runの道具・測定script・自己検査・確率・dry runは`<scratch>/step3f-b12-design/`（以下、記録の場所はここからの相対path）に置いた。3f-b11の記録は`b11:`、3f-b10の記録は`b10:`を付けて示す。

### 2つの登録案（C1）

どちらも`scripts/accept.py`の`THRESHOLDS`にあり、`variants/width/`と`variants/exact/`にそれぞれの`registration.json`と`registration.sha256`がある（2026-11-17T22:59:33Z、UTC）。統計、単位、runの値、判定の種類、交わりの規則は3f-b11のまま変えない。変わるのは閾値と、その根拠だけである。

| 判定                                        | 決定で認めた値 | Exactの閾値 | Widthの閾値 |
| ------------------------------------------- | -------------- | ----------- | ----------- |
| in-process `16/wasm-inprocess`、impl/before | 1.006          | 1.0060      | 1.0100      |
| in-process `64/wasm-inprocess`、impl/before | 1.002          | 1.0020      | 1.0060      |
| in-process 16・64、impl/norm（破棄の費用）  | 1.011          | 1.0110      | 1.0150      |

Widthの数は3f-b11と同じだが、3f-b11の「認めた値 ×（1 + Δ）」という導出は撤回する。数はownerが承認した固定値そのもので、Δ、検出力、どちらの側の散らばりからも導かない。

Uの意味も言い直す。Uは片側97.5%の信頼上限で、同じ手順を繰り返したとき、そうして作った上限の97.5%以上が真の比を下回らない。U ≤ 閾値で受理する規則は、真の比が閾値を超える実装を受理する頻度を、判定ごとに2.5%以下に抑える。保存した1回の系列について「真の比が閾値以下である確からしさ」を述べるものではない。

2つの案が実際に意味すること。Exactでは、認めた値そのものが上限になる。(a4)と同じ速さの実装でも、測定の誤差の幅だけ上限をはみ出して見えるので、結果はほとんど毎回inconclusiveになり、ownerが測った数値を見て決めることになる。(a4)より0.2%速い実装でも受理は74%・40%にとどまる。Widthでは、各上限が認めた値より0.4ポイント上にあるので、(a4)と同じ実装はほぼ確実に受理されるが、認めた値より最大約0.4%遅い実装も通り得る。

保存した3f-b10の2系列から、27 processを復元抽出で20,000回引いて求めた（`scripts/probability.py`、`records/probability.json`。1つのprocessの全armを一緒に引く。乱数の種は3f-b11と同じで、Widthの値は3f-b11の表と一致した）。(a4)の対数比に一様な遅さを足した行である。値はaccept / inconclusive / slowerの割合。

| (a4)に足した遅さ | Width 1系列目          | Width 2系列目         | Exact 1系列目         | Exact 2系列目          |
| ---------------- | ---------------------- | --------------------- | --------------------- | ---------------------- |
| 0（(a4)と同一）  | 0.997 / 0.003 / 0      | 0.970 / 0.030 / 0     | 0.056 / 0.934 / 0.009 | 0.0007 / 0.931 / 0.068 |
| +0.1%            | 0.955 / 0.045 / 0      | 0.802 / 0.198 / 0     | 0.002 / 0.877 / 0.121 | 0 / 0.516 / 0.484      |
| +0.2%            | 0.728 / 0.272 / 0      | 0.375 / 0.625 / 0     | 0 / 0.400 / 0.600     | 0 / 0.054 / 0.946      |
| +0.3%            | 0.320 / 0.680 / 0.0003 | 0.043 / 0.953 / 0.004 | 0 / 0.019 / 0.981     | 0 / 0.001 / 0.999      |

どちらの案でも、認めた値をどれか1つの判定で1%超える実装（その判定の列を認めた値 × 1.01へ置き直した系列）は、両系列・4判定とも20,000回の抽出で受理0回、slower 20,000回だった。0回でも誤受理の率が0だとは言えず、Monte Carloの誤差だけでの片側95%上限は約0.00015である。どの表も、統合後の比が保存した系列の同時分布に従うという前提つきの見積りで、平均だけでなく分散・相関・測定過程が変われば変わる。

### ownerの2つ目の決定（2026-11-18）

ownerはWidthを選んだ（「bで良い。トレードオフを明示的に示してほしい」）。1つ目の決定（選択肢A、3f-b10で測った(a4)の費用1.006・1.002・1.011を許容幅として認める）は変わらない。2つ目の決定は、その上に置く0.4ポイントの上乗せを明示的に承認するもので、承認した閾値は1.0100・1.0060・1.0150である。記録は`records/decision.json`にある。

実装の測定はWidthの登録で行う。測定を始める`begin`は、この決定の記録がないとき、記録がWidth以外を名指すとき、記録の承認した閾値が登録と1つでも違うとき、登録したファイルのどれかがsha256と違うときに拒否する。Exactの登録はこの道具では使えず、選ばれなかった案の記録として残る。

### トレードオフ

ownerとpull requestの説明がそのまま引用できるように書く。数はすべて、保存した3f-b10の2系列（上流の統合の前）から、登録したWidthの閾値で求めた。

Widthで得るもの。

- 許容幅は測定の前に固定してある（2026-11-17T22:59:33Z、`variants/width/registration.sha256`）。測定の後で閾値を動かすことはできない。
- (a4)と同じ速さの実装は、2つの系列でそれぞれ99.7%・97.0%の確率で受理される。
- Exactなら、同じ実装の受理は約5.6%・0.07%しかなく、9割以上がinconclusiveで、0.9%・6.8%はslowerと判定される。

Widthで払うもの。

- 閾値は、認めた値より0.4ポイント上にある。16の合計は1.0100（認めた値1.006）、64の合計は1.0060（1.002）、破棄の費用は1.0150（1.011）。
- (a4)より遅い実装も受理され得る。0.1%遅ければ95.5%・80.2%、0.2%遅ければ72.8%・37.5%、0.3%遅ければ32.0%・4.3%の確率で受理される。レビューが作った反例（保存した(a4)を0.2%遅くした系列。16の合計の下側限界1.0061は認めた1.006を上回る）も受理される。
- 認めた値をどれか1つの判定で1%超える実装は、20,000回の抽出で一度も受理されなかった（両系列・4判定とも）。
- 判定は片側の上側限界で行うので、通り得る真の遅さの上限は各閾値（1.0100・1.0060・1.0150）であり、実際に通る実装の真の比は閾値より少し下にとどまる。これは閾値として述べる上限で、測って確かめた値ではない。

どちらの案でも判定しないもの。

- nativeの実時間（usableのnative-jsonlとcriterion全体）。
- 新しいprocessでのwire経路。
- Expression・Branch・TemporalFormulaの破棄経路。in-processのharnessは有限modelの`Value`の破棄と`normalize`だけを通る。drop_budgetの10件はこれらの経路の破棄中の確保回数を要求するが、時間は要求しない。
- usableとcriterion。どちらも記録だけである。

受入れが言えるのは、in-processの4つの判定がすべて閾値以下だった、ということだけである。

### C2: 計算の入力検査

`scripts/accept.py`の`block_differences`は、比べる2つのarmについて次を確かめ、どれかが崩れれば計算を拒否する。

- familyの全行が`run`を持ち、その値が1つだけで、判定するrunのidと等しい。
- 各armのblock idの集合が、計画した0..n−1（in-process 27、usable 129、criterion 40）と等しい。
- 各(path, block, arm)の値の数が登録した数（in-process 1、usable 2、criterion 2）とちょうど等しい。
- 値が正の有限数で、block idが整数である。判定pathやbaselineのarmが欠けていれば拒否する（3f-b11のまま）。

記録familyの拒否はその場で報告し、判定を変えない（3f-b11のまま）。統計は変えていない。保存した(a4)の3f-b10の限界をbit単位で再現し、Widthのdry runは3f-b11のdry runと同じ判定を返した。

### C2: converter

`scripts/convert.py`はrunのdirectoryを読む。

- in-process: `inprocess/`の中身がちょうど`proc-0.json`〜`proc-26.json`でなければ拒否する（ほかのファイル1つでも拒否）。各ファイルの`runId`がrunのid、`processIndex`がファイル名の番号、`rounds`が登録した600、`arms`が登録した`before, placebo, norm, impl`、sizeが16と64、各armの標本がちょうど600個の正の有限数であることを要求する。process idの重複は、ファイル名との照合より前に、重複として拒否する。runの値は3f-b10と同じ600標本の中央値（`ordered[300]`）である。
- usable: `usable/`の中身が、計画した鏡像blockの名前`b{block}-p{位置}-{arm}.json`（129 block × 8 run）とちょうど等しいことを要求し、各ファイルの`runId`と、`cwd`がそのarmのartifactであることを確かめる。
- criterion: `criterion/ledger.jsonl`が計画した順序（3 armの前半をblockごとに回し、後半を逆順にする、40 block × 6 run）と1行ずつ等しいことを要求し、計画にないbaselineのdirectoryを拒否する（criterion自身の作業用`new`は読まない）。

### C2: runの状態と封印

runの状態は`scripts/measurement_run.py`の1か所で決まり、ファイルの有無から導く（`run.json`だけなら開いている、`verdict.json`があれば封印済み、`report.json`があれば報告済み）。どのファイルも一度だけ作る（既存なら書込みが失敗する）。

| 状態        | 出来事                                              | 行き先                                                   |
| ----------- | --------------------------------------------------- | -------------------------------------------------------- |
| （runなし） | `begin`（決定の記録と登録ファイルのsha256が一致）   | 開いている。新しいrun id（UTC時刻と乱数12桁）            |
| 開いている  | in-processの1 process（`admit inprocess`）          | 開いている                                               |
| 開いている  | usableまたはcriterionの1 run                        | 拒否                                                     |
| 開いている  | `seal`                                              | 封印済み。判定（または入力の拒否）、時刻、全入力のsha256 |
| 封印済み    | in-processの観測、2回目の`seal`                     | 拒否                                                     |
| 封印済み    | usableまたはcriterionの1 run（`admit`が封印を検証） | 封印済み                                                 |
| 封印済み    | `report`                                            | 報告済み                                                 |
| 報告済み    | どの観測も、2回目の`report`も                       | 拒否                                                     |

各状態で常に成り立つこと。開いているrunには判定がない。封印済みのrunでは、`verdict.json`のsha256が`verdict.sha256`と一致し、`inprocess/`の中身と`inprocess-observations.jsonl`が封印に記録したファイルの集合・sha256と一致する（1つ足しても変えても、次の`admit`が拒否する）。報告済みのrunの判定は封印の判定と等しく、`report`は封印した観測から判定を計算し直して一致しなければ拒否する。

入力が拒否されたrun（processの欠落など）も、`refused`という判定として封印する。そのrun idで観測を足して完成させることはできない。ownerへ返し、次のrunはownerの指示で新しいrun idから始め、報告にはすべてのrun idとその封印した結果を並べる。

測定は`scripts/measure-run.sh`の段で行う。`begin <runs-root> <artifacts-root>`、続いて`all <run>`が counters → inprocess → seal → usable → criterion → report を順に走らせる。inprocessの各processの前に`admit inprocess`、usableとcriterionの各runの前に`admit`（封印の検証を含む）を呼び、拒否なら止まる。sealが`refused`を封印したときも止まる。harness（`scripts/inprocess.mjs`、`scripts/usable-run.mjs`）は3f-b10のものと、run idとartifactの場所を引数に取ることと、中央値を表示しないことだけが違う。これらのscriptもWidthの`registration.sha256`に含まれ、`begin`が照合する。`admit`の照合は各runの計測区間の外（runとrunの間）で行い、計測値には入らない。その時間は測っていないので、全体の時間の見積りには含めていない。

### 自己検査

`scripts/accept_selftest.py`は116/116で通った（`records/accept-selftest.txt`、sha256は`records/selftest.sha256`）。

- 3f-b10・3f-b11から引き継いだ19件。統計の再現、欠落・不足・非正値・不均衡・判定pathやbaselineの欠落の拒否、記録familyの拒否が判定を変えないこと。
- C2の計算18件。レビューの連結入力（保存した2系列の(a4)を1.0027倍）は、各系列だけならどちらもinconclusiveで、3f-b11の計算は連結をacceptにする（レビューの再現）。新しい計算は、run idのない元の形、1つのrun idにそろえた形（一値の検査で拒否: 各blockに2つ）、系列ごとのrun idを残した形（1 runの検査で拒否）のどれも拒否し、Width・Exactのどちらでも拒否する。全armを2倍にした均衡な重複（3f-b11はaccept）、process id 1..27（3f-b11は拒否しない）、28 process、1つだけ計画外のid、別のrunの観測も拒否する。usableで1 runや4 run（2系列の連結）、criterionで3 runのblockは、その場で拒否され、判定は変わらない。
- C2のconverter 20件。保存した3f-b10の27ファイル（armを`before, placebo, norm, impl`へ絞り、a4をimplと呼び替え、run idを付けたもの。標本は変えない）は、保存したin-processの観測と1値も違わずに変換される。レビューの名前替え入力（2系列目を`proc-k-old.json`として同じdirectoryへ置く）は、3f-b10のconverterなら54 processを読み、3f-b11の計算がacceptにするが、新しいconverterは拒否する。process idの重複、ファイル名と違うid、`rounds`ごと500にした標本（3f-b10のconverterは通す）、1 armだけ599の標本、欠けたファイル、別のrunのファイル、違うarm、計画外のファイルを拒否する。usableとcriterionでも、計画した名前と順序だけが通り、余分なrun、別のartifactのrun、計画外のbaseline、ledgerの余分な行を拒否する。
- runの状態17件。決定の記録がない、Exactを名指す、承認した閾値が違う、登録ファイルのsha256が違う、のどれでも`begin`は拒否する。`begin`ごとに新しいrun idができる。封印は判定・時刻・28入力のsha256を書き、封印済みのrunはin-processの観測と2回目の封印を拒否し、封印後に足したファイルや変えたファイルは次の`admit`で拒否される。`report`は封印の判定を保ち、欠けたusableとcriterionをその場で報告し、報告済みのrunはすべてを拒否する。入力が拒否されたrunは`refused`として封印され、後から完成させられない。
- 測定scriptの段6件。harnessを保存したprocessファイルを書くだけの代役に替えて`measure-run.sh`を走らせ、`begin`、27ファイルの測定、封印（2系列目の(a4)でaccept）、封印済みのrunでのin-process再測定の拒否（ファイルは増えない）、封印のないrunでのusableの拒否（ファイルはできない）、timelineで封印がin-processの後にあることを確かめた。benchmarkは走らせていない。
- C1のWidth 18件。閾値が述べた数そのものであること、`registration.json`と`registration.sha256`が計算と一致すること。承認した幅の帰結として、レビューの保存反例（(a4) × 1.002）と、全判定を認めた値の0.2%上に置いた雑音なしの系列はacceptになる。保存した(a4)の2系列はaccept。認めた値で生成した系列は40中39でaccept。認めた値より1%遅い実装は、雑音なし・40 seed・保存系列の1.01倍・1判定ずつのどれでもslower。beforeの外れ値を広げても閾値も判定も変わらない。64の閾値ちょうどの実装は200中7でaccept。
- C1のExact 18件。閾値が認めた値そのものであること。レビューの2つの反例はslower。保存した(a4)の2系列はinconclusive。認めた値で生成した系列は40中0でaccept。beforeと等しい実装は40中25でaccept、slowerは0。1%遅い実装の各検査はWidthと同じくslower。

負の対照（`scripts/negative_controls.py`、`records/negative-controls.txt`）。登録したファイルを変えずにメモリ上で1か所ずつ戻し、同じ自己検査を走らせた。3f-b11の入力検査に戻すとC2の計算の12件が落ちる（104/116）。3f-b10のconverterに戻すとconverterの9件が落ちる（107/116）。`admit`を常に許可にするとrunの状態の5件が落ちる（111/116）。Width の閾値をExactの値に替えると、承認した幅の帰結の2件を含む11件が落ちる（105/116）。

### dry run（受入れの証拠ではない）

保存した3f-b10の系列（統合前）に、それぞれの系列名をrun idとして付け、Widthの登録とExactで再計算した（`scripts/dryrun.py`、`records/dryrun.txt`・`records/dryrun.json`）。

| arm     | 系列 | Width        | Exact        | Exactで閾値を超えた判定                                                                |
| ------- | ---- | ------------ | ------------ | -------------------------------------------------------------------------------------- |
| a4      | 1    | accept       | inconclusive | 16 before比 [1.0041, 1.0075]                                                           |
| a4      | 2    | accept       | inconclusive | 16 before比 [1.0046, 1.0074]、64 before比 [1.0002, 1.0036]、64 norm比 [1.0090, 1.0131] |
| p3n     | 2    | inconclusive | slower       | 16 before比 [1.0063, 1.0082]                                                           |
| p1n     | 1    | slower       | slower       |                                                                                        |
| p2n     | 1    | slower       | slower       |                                                                                        |
| b8      | 1    | slower       | slower       |                                                                                        |
| placebo | 1    | accept       | inconclusive | 64 norm比 [1.0085, 1.0113]                                                             |
| placebo | 2    | accept       | inconclusive | 64 before比 [0.9996, 1.0034]、64 norm比 [1.0085, 1.0127]                               |

Widthの判定と、判定・記録の比と限界は、40の判定値を含めてすべて3f-b11のdry runと同じ値である。placeboは`normalize`のin-place化を持たないので、そのnorm比は破棄の費用ではない。

### 3f-b11・3f-b10から変えないもの

- 破棄の設計（(a4)、予算B = 32、`Value`の高さ2までのglue委任、境界）、`normalize`のin-place化を破棄の費用と分けて報告すること、テスト段で足す21件とその赤・感度の記録、凍結ファイルの数（テスト段の後で71）、stackの上限とbyte数の見積り。
- 統計（鏡像blockとprocessを単位とする対数比、片側97.5%の上下限、交わりで受理、Berger 1982）、判定4件（in-processのsize 16・64のbefore比とnorm比）、記録だけのfamily（in-processのplacebo比・norm比、usable 8本、criterion 14本）、blockとprocessの数（27・129・40）。27は3f-b10の登録値のままで、3f-b11の「0.4%を24 processで見分けられる」はnの説明であり、許容幅の根拠には使わない。
- armの定義（統合した木から作るimpl・before・norm・placebo）、上流main（d147550c）を統合してから測ること、順序（build → 命令数・確保回数 → in-process → 封印 → usable → criterion）、時間の見積り（約2時間20分、封印は開始から約16分）。
- 判定の種類と、inconclusiveをGreenにしないこと、再測定で判定を変えないこと、記録familyの拒否が判定を変えないこと。
- 番号（統合で0048へ移る）と、`Value::normalize`以外の深い値の再帰がb8の残課題であること。

### 3f-b11から変えたもの

- 閾値の根拠。数は同じだが、Δからの導出ではなく、ownerの2つ目の決定による承認だけを根拠にする。Exactは選ばれなかった案として残す。
- 計算の入力検査、converter、run id、runの状態と封印の段（上のとおり）。3f-b11の「測定scriptは3f-b10のものをarmと出力先だけ変えて使う」は、`measure-run.sh`と2つのharnessの写しに替わった。
- 3f-b11の「(a4)と見分けられない灰色の帯」は、ownerが承認した0.4ポイントの幅と言い直す。「確率1で拒否」と「97.5%の確からしさ」は上の言い方に替える。
- 判定しない範囲にExpression・Branch・TemporalFormulaの破棄経路を加え、「同じRustの破棄のsourceがin-processで判定される」を撤回する。

### 残すもの

- 受入れが言えるのはin-processの4判定だけである。nativeの実時間、新しいprocessでのwire経路、Expression・Branch・TemporalFormulaの破棄経路は判定せず、usableとcriterionは記録だけである。
- 承認した幅の内側では、(a4)より約0.2〜0.3%遅い実装も通り得る。1%遅い実装は20,000回の抽出で一度も通らなかったが、これは保存した系列の分布を前提にした見積りである。
- usableとin-processの差約1.5%を、変更の仕事でないと因果で分けたわけではない。usableが配置で最大2%動くことは、証拠の限定として扱う。
- 統合や系列の違いで比が0.2%以上動けば、(a4)と同じ実装もinconclusiveかslowerになり得る。その場合は再測定せずownerへ返す。
- `Value::normalize`以外の深い値の再帰（Clone・PartialEq・Hash・Debug・Serialize、`Arena::data`）はb8の残課題のままである。

## step 3f-b13 design: 測定runの状態機械、task id、t限界が述べられる範囲

改訂: 2026-11-18 初版（前提レビュー前の草案）。3f-b12の前提レビューはC1を閉じ、NO-GO（C2-A・C2-B・P1）を返した。3f-b12からの変更と理由は次のとおり。

- runの状態を、ファイルの有無から導くのをやめ、1つの`state.json`に置いた（C2-A・C2-B）。`state.json`は一時ファイルへ書いてfsyncし、renameで置き換える。3f-b12は`run.json`・`verdict.json`・`report.json`の有無で状態を決めていたので、書込みの途中で止まると、観測のある開いたrun、digestのない封印済みrun、不完全なJSONの報告済みrunという、表にない状態が残ったため。
- 測定の前にtaskの開始を`(run id, family, task id)`として永続化し、非0終了・開始して完了しないtask・同じtaskの2回目の開始でrunを終端のfailedにした（C2-A）。3f-b12の`admit`は開始を記録せず、失敗したprocess 0を同じrunで測り直してacceptまで進めた（試行列0, 0, 1, …, 26）ため。
- 封印にsealing、報告にreportingという明示的な状態を足し、snapshotと報告を一時の場所に作って1回のrenameで公開してから状態を進めるようにした（C2-B）。中断はsealingかreportingとして残り、凍結した同じ中身を完成させるか、failedへ進むかしかない。報告にもreportingを足したのは、公開とstate.jsonの書込みの間の中断で「report.jsonはあるがrunは報告済みでない」状態が残り、その間に観測を足せたためである。
- 入力のlistingはlstatで種類（file・directory・symlink・other）を記録し、正規ファイルだけをhashする（C2-B）。3f-b12は`inprocess/backup/`をファイルとしてhashしようとしてIsADirectoryErrorで止まり、拒否すべきrunを封印できなかったため。
- usableのharnessは自分の出力にtask id（`b{block}-p{位置}-{arm}`）を書き、converterは計画したファイル名との一致と重複を確かめる。criterionのbaseline名もtask idにした（レビューの非ブロッキング注記）。3f-b12のusable出力はrun idとcwdしか持たず、同じarmの別の観測を別の計画名へ複製すると2回の測定として数えたため。criterionの`ledger.jsonl`は、state.jsonの完了記録（計画順の先頭からの連続）に置き換えた。
- t限界の文言を、i.i.d.正規モデルの下での片側97.5%上限に限り、実際の測定過程での被覆率と非正規27標本での誤受理の上限は未証明、閾値は観測したUの境界で真の比の上限ではない、と改めた（P1）。登録をこの文言で取り直した。3f-b12の本文1354行・1390行とregistration.jsonの`coverage`は、方法が支えない無条件の保証を述べていたため。
- 報告に`judgement`を足した。同じrootで最初に判定（accept・inconclusive・slower）を封印したrunの判定である。新しいrunでの再測定が判定を変えないこと（3f-b10からの規則）を、新しいrunを許す手順の下でも機械的に保つため。
- 自己検査を116件から189件にし、負の対照を4つから8つにした。

本番ソースとテストは変えていない。状態機械・converter・driver・harnessの写し・自己検査・確率・dry runは`<scratch>/step3f-b13-design/`（以下、記録の場所はここからの相対path）に置いた。3f-b12の記録は`b12:`を付けて示す。統計（`scripts/accept.py`・`scripts/tdist.py`）とin-processのharness（`scripts/inprocess.mjs`）、ownerの決定の記録（`records/decision.json`）はb12とbyte単位で同じである。

### runの状態機械

runの状態は`state.json`の`state`だけで決まる。ほかのファイル（観測、`sealed/`、`report.json`、一時の`*.partial`）の有無から状態を導くことはない。一時の名前は読まれず、次の公開が上書きする。

| 状態             | state.jsonが持つもの                                                        | 常に成り立つこと                                                                                                                                                        |
| ---------------- | --------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| open             | 各familyの完了task（task id、出力のsha256、開始・完了時刻）                 | 走っているtaskはない。各familyの完了taskは計画の先頭からの連続で、それぞれ出力のsha256を持つ。完了記録と違う出力や記録にない項目は、封印の時に入力の拒否になる          |
| measuring(task)  | openのもの + countersかinprocessのtask 1つと開始時刻                        | そのtaskの開始は測定の前に永続化されている。その出力はまだ観測として数えない                                                                                            |
| sealing          | + 凍結したin-processのlisting（名前・種類・正規ファイルのsha256）と封印時刻 | taskを受け付けない。`inprocess/`の中身は凍結したlistingと等しい。`sealed/`は無いか、凍結したlistingから決定的に作るsnapshotとbyte単位で等しい                           |
| sealed           | + snapshotのsha256と判定                                                    | `sealed/`は`digests.sha256`・`inputs.json`・`observations.jsonl`・`verdict.json`の4ファイルで、digestが一致する。`inprocess/`は凍結したまま。判定はsnapshotの判定である |
| recording(task)  | sealedのもの + usableかcriterionのtask 1つと開始時刻                        | sealedと同じ。そのtaskの開始は測定の前に永続化されている                                                                                                                |
| reporting        | sealedのもの + 報告時刻                                                     | taskを受け付けない。`report.json`は無いか、封印したrunと完了taskから決定的に作る報告とbyte単位で等しい                                                                  |
| reported（終端） | + `report.json`のsha256                                                     | `report.json`がそのsha256と一致する。読む以外のすべての出来事を拒否する                                                                                                 |
| failed（終端）   | 元の状態、理由、task、封印済みだったなら判定                                | 読む以外のすべての出来事を拒否する。封印済みの判定は、元の状態がsealed・recording・reportingのときだけ持ち、読める                                                      |

出来事と行き先。「拒否」は何も測らず何も書かない（state.jsonがbyte単位で同じ）。failedは測定の過程が壊れたとき（taskの失敗・中断・二重開始、runの記録の外で作られた出力）と、凍結・公開した中身が変わったときで、runを終える。in-processの入力が完了taskの出力そのものでないときは、3f-b12のまま判定`refused`として封印する。

| 状態              | 出来事                                                                                                                        | 行き先                                                               |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| （runなし）       | begin（決定の記録、登録ファイルのsha256、同じrootの以前のrunがすべてreportedかfailed）                                        | open。runのdirectoryを`<run id>.partial`で作り、renameで公開する     |
| open              | start(f, t)。fはcountersかinprocess、tはfの次の計画task、tの出力はまだ無い                                                    | measuring(t)                                                         |
| open・sealed      | start(f, t)。tがすでに開始された（二重開始）                                                                                  | failed                                                               |
| open・sealed      | start(f, t)。tの出力が開始の前にある                                                                                          | failed                                                               |
| open・sealed      | start(f, t)。fがその状態で受け付けるfamilyでない、tが計画外、tが次の計画taskでない                                            | 拒否                                                                 |
| measuring(t)      | complete(t)。出力が正規ファイル                                                                                               | open（出力のsha256を完了として記録）                                 |
| measuring(t)      | complete(t)。出力が無いか正規ファイルでない                                                                                   | failed                                                               |
| measuring(t)      | ほかのすべての出来事（fail、start、seal、report、別taskのcomplete）                                                           | failed（tは開始されて完了していない）                                |
| open              | seal                                                                                                                          | sealing（`inprocess/`のlistingと封印時刻を凍結）。続けて公開を試みる |
| sealing           | seal（公開）。凍結した中身が変わらず、snapshotを`sealed.partial/`に作り1回のrenameで`sealed/`へ公開した、または公開済みが同じ | sealed                                                               |
| sealing           | seal（公開）。公開済みの`sealed/`が凍結したlistingのsnapshotと違う                                                            | failed                                                               |
| sealing           | start、complete、report                                                                                                       | 拒否                                                                 |
| sealed            | start(f, t)。fはusableかcriterion（ほかの規則はopenと同じ）                                                                   | recording(t)                                                         |
| recording(t)      | complete(t)／完了しない、またはほかの出来事                                                                                   | sealed／failed（measuringと同じ）                                    |
| sealed・reporting | seal                                                                                                                          | 拒否（判定は1回だけ封印する）                                        |
| sealed            | report                                                                                                                        | reporting（報告時刻を凍結）。続けて公開を試みる                      |
| reporting         | report（公開）。報告を`report.json.partial`に作りrenameで公開した、または公開済みが同じ                                       | reported                                                             |
| reporting         | report（公開）。公開済みの`report.json`が違う、または封印した観測が封印の判定を再現しない                                     | failed                                                               |
| reporting         | start、complete                                                                                                               | 拒否                                                                 |
| 終端でないすべて  | fail（harnessの非0終了、またはoperatorがrunを終える）                                                                         | failed                                                               |
| 終端でないすべて  | 出来事の前の検査で不変条件が崩れている（登録ファイルの変化、凍結した入力の変化、公開済みsnapshotの変化）                      | failed                                                               |
| reported・failed  | 読む以外のすべて                                                                                                              | 拒否                                                                 |
| すべて            | 読む（`status`）                                                                                                              | 変わらない（何も書かない）                                           |

遷移を決めるのは`scripts/measurement_run.py`の`advance(run, event)`だけで、これはファイルを読まず書かない。`apply`がrunのlockの下でstate.jsonを読み、不変条件を検査し（崩れていれば出来事を`violation`に替える）、観測（出力のsha256、listing、公開の結果）を出来事に入れて`advance`を呼び、状態が変わったときだけstate.jsonを置き換える。snapshotと報告の公開は`seal`・`report`の2回目の`apply`の中で行い、その結果が`published`・`report-published`の出来事になる。封印時刻と報告時刻は状態に凍結してあるので、公開の中身は凍結した入力だけから決まり、中断の後に作り直しても同じbyteになる。

task idは、countersが`all`（36回の計数を1つのtaskとする）、in-processがprocessの番号`0`〜`26`、usableとcriterionが`b{block}-p{位置}-{arm}`である。run idはstate.jsonの`runId`で、開始の記録はmeasuring・recordingの状態そのものである。

### C2-A: taskの開始・完了・失敗

`scripts/measure-run.sh`は、各taskについて`start` → harness → `complete`を呼ぶ。harnessが非0で終わると`fail`を呼んでrunをfailedにし、終了コード4で止まる。driver自身が殺されたときはstate.jsonがmeasuring(t)のまま残り、次の出来事がrunをfailedにする。startは次の計画taskだけを受け付けるので、完了taskは常に計画の先頭からの連続で、段を走らせ直すと最初のstartが二重開始としてrunを終える。

欠けた観測は新しいrunでだけ測る。`begin`は同じrootの以前のrunがすべてreportedかfailedであることを要求し（開いたままのrunはoperatorが`fail`で終える）、以前のrunのid・終端の状態・判定・理由を`previousRuns`として記録する。報告は`previousRuns`を並べ、`judgement`に、同じrootで最初にaccept・inconclusive・slowerを封印したrunの判定を書く。封印の前にfailedになったrunは判定を持たず、harnessは中央値を表示しないので（3f-b12のまま）、測り直しは判定を見て選ぶことにならない。封印の後でfailedになったrunの判定は残り、後のrunの判定はそれを置き換えない。新しいrunはownerの指示で始める（3f-b12のまま）。

自己検査（`records/accept-selftest.txt`のC2-A）:

- レビューの試行列。3f-b12のdriverに同じ代役（最初の試行だけ終了コード17）を当てると、試行列0, 0, 1, …, 26でacceptに達する（レビューの再現）。新しいdriverでは、process 0の失敗でrunがfailed（理由`harness exit 17`）になり、2回目のin-processの段とsealは拒否され、試行列は0だけで、判定はなく`sealed/`もできない。同じ列をmoduleの呼出しでも確かめた。
- 開始と完了の間のcrash。process 2のharnessがdriverをSIGKILLすると、state.jsonはmeasuring(inprocess 2)のまま残り（完了は0と1）、次の段の最初の出来事がrunをfailed（`inprocess task 2 was started and not completed before start inprocess 0`）にし、試行は0, 1, 2から増えない。moduleでは、未完了のtaskの後のseal、同じtaskの再start、recording中のtaskの後のreportがrunをfailedにし、最後の場合も封印した判定（accept）は読める。
- 二重開始。27 processを測った後にin-processの段を走らせ直すと、harnessを1回も呼ばずにrunがfailed（`second start of inprocess task 0`）になり、出力は変わらない。
- そのほか。開始の前に出力があるtask、出力なしやdirectoryの出力での完了、countersの失敗はrunをfailedにする。failedのrunはすべての出来事を拒否し、state.jsonは変わらない。failedのrunの後のbeginはそれを`previousRuns`に記録し、空の完了記録から始まる。封印の後でfailedになったrunの判定（slower）は、次のrunの判定（accept）の報告でも`judgement`として残る。

### C2-B: 封印と報告の公開

封印は2回の`apply`である。1回目はopenからsealingへ進み、`inprocess/`のlistingと封印時刻を凍結する。この時点からstartは拒否される。2回目は、凍結したlistingと`inprocess/`が等しいことを確かめ、converterと判定からsnapshot（`inputs.json`は凍結したlisting、`observations.jsonl`、`verdict.json`、それらのsha256を並べた`digests.sha256`）を`sealed.partial/`に書いてfsyncし、`sealed/`へ1回のrenameで公開し、その後でstate.jsonをsealedにする。報告も同じで、sealedからreportingへ進んで報告時刻を凍結し、報告を`report.json.partial`に書いてrenameで公開し、その後でreportedにする。

入力のlistingはlstatで項目の種類を記録し、正規ファイルだけをsha256で記録する。directory、link、pipeは名前と種類だけを残し、開かない。計画した名前が正規ファイルでないとき、計画外の項目があるときは、converterが種類を付けて拒否し、そのrunは判定`refused`として封印される。

自己検査（同じファイルのC2-B）。レビューの4つの場合と、公開とstate.jsonの書込みの間の中断を、書込み関数への中断の注入で再現した。

| 中断の位置                          | 3f-b12（レビュー）                                          | 3f-b13                                                                                                                                                        |
| ----------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| observationsを書いた後、verdictの前 | open。admitは許可、再sealはFileExistsError                  | sealing、`sealed/`は無い。in-process・usableのstartとreportは拒否。次のsealが凍結した同じsnapshotを完成させ、中断なしのsealとbyte単位で同じ                   |
| verdictを書いた後、digestの前       | sealed。verify・reportがFileNotFoundError、再sealは拒否     | 同上                                                                                                                                                          |
| snapshotの公開の後、state.jsonの前  | （該当なし）                                                | sealing、`sealed/`は完全。次のsealが同じsnapshotであることを確かめてsealed。`sealed/`を変えてあればfailed                                                     |
| reportの書込み途中                  | reported、`report.json`は不完全なJSON                       | reporting、`report.json`は無い。startとsealは拒否。次のreportが完全なJSONを公開し（中断なしの報告と同じbyte）、reported                                       |
| reportの公開の後、state.jsonの前    | （該当なし）                                                | reporting。次のreportが同じ報告であることを確かめてreported。`report.json`を変えてあればfailed                                                                |
| `inprocess/backup/`（directory）    | IsADirectoryErrorで封印できず、openと空のobservationsが残る | sealed、判定refused、拒否理由は`unplanned ['backup (directory)']`。凍結したlistingは`{"name": "backup", "kind": "directory"}`を持ちhashしない。reportまで進む |

加えて、計画した名前のsymlinkとpipeは種類で拒否されて開かれない（pipeを開くとblockするので、5秒の制限を付けて確かめた）。sealingの間に足した観測（`proc-27.json`）は入らず、runはfailedになる。封印の後で凍結した入力や`sealed/`を変えると、次の出来事がrunをfailedにし（3f-b12は次のadmitを拒否するだけで、戻せば続けられた）、封印した判定は読める。statusはそれを書込みなしに報告する。

### usableとcriterionのtask id

`scripts/usable-run.mjs`は3つ目の引数のtask idを出力の`taskId`に書く（3f-b12との差はこの2行だけ）。driverはtask idを渡し、出力を`usable/<task id>.json`に置く。converterは、計画した1,032の名前がちょうど正規ファイルとしてあること、内容の`taskId`に重複がないこと（ファイル名との照合より前）、`taskId`がファイル名の計画taskと等しいこと、run id、cwd、各出力が完了時に記録したsha256と等しいことを確かめる。criterionはbaselineの名前をtask idにし（`--save-baseline b{block}-p{位置}-{arm}`）、logも`criterion/logs/<task id>.log`に置く。converterは計画外のbaseline（3f-b12の`impl-3`の形も）、完了していないtask、完了時と違う出力を拒否する。usableとcriterionの拒否は3f-b12のまま記録familyの中で報告し、判定を変えない。

自己検査:

- レビューの複製（`b0-p8-before.json`を消して`b0-p1-before.json`をその名前へ複製）。3f-b12のconverterは1,032ファイルをすべて読み、16/wasm-wireをn = 129として比べる（レビューの再現）。新しいconverterは、複製を完了した出力として記録した場合（driverが誤った出力を別の計画名へ書く場合）も、元の完了を記録した後で複製した場合も、`duplicate task ids ['b0-p1-before']`で拒否する。
- 同じblock・同じarmの前半と後半（p1とp8）の入替えは、`taskId`とファイル名の不一致で拒否する。
- 保存記録の全lifecycle（27 process、封印、usable 1,032 task、criterion 240 task、報告）の後に同じ複製をすると、報告はusableをその場で拒否し、判定（accept）は変わらない。複製しない同じrunの報告は、in-process 27、usable 129、criterion 40のnで記録する。
- criterionでは、完了した後で別の計画taskのbaselineへ複製した出力を拒否する。driverの段では、usableの出力がそれぞれ自分のtask idを持ち、criterionのbaselineとlogがtask idの名前になる（どちらも代役のharnessを途中で失敗させ、runがfailedになり、封印した判定が残ることも確かめた）。

### P1: Uが述べられること

3f-b12の「Uの意味も言い直す」（本文1354行）を次で置き換える。閾値、式、判定の種類は変えない。

- U = exp(d̄ + t(0.975, 26)·s/√27)が片側97.5%の上側信頼限界であるのは、判定ごとの27個のprocessの対数比d_jが独立に同じ正規分布に従う、というモデルの下だけである。そのモデルの下では、同じ手順を繰り返したとき限界の97.5%以上が真の比（幾何平均の比）以上にあり、U ≤ 閾値で受理する規則が、真の比が閾値を超える実装を受理する割合は判定ごとに2.5%以下である（4判定の交わりの規則もこのモデルの下でのことである）。
- 実際の測定過程での被覆率と、正規でない27標本での誤受理の上限は、証明していない。レビューの反例では、processの比が確率0.99で1.005、0.01で1.12のとき、真の比は1.00609で64の閾値1.0060を超えるが、27個がすべて1.005になる確率0.762で、登録した計算は受理する。
- 閾値は観測から求めたUの境界であって、真の比の上限ではない。真の比が閾値を超える実装も受理され得て、それは正規モデルの下でも起こる（真の比1.0061、対数の標準偏差0.003の27標本の1つでU = 1.0058006583、受理）。
- 変えないもの。bootstrapの表は保存した2系列の同時分布を前提にした値で、正規性を仮定しない。「20,000回の抽出で受理0回」と、Monte Carloの誤差だけでの片側95%上限約0.00015もそのままである。これらは保存した分布の下での数で、実際の測定過程での誤受理率の上限ではない。

自己検査（P1）は、2つの登録の`coverage`がこの3点を述べ、3f-b12の無条件の文を含まないこと、レビューの2つの反例を登録した計算がacceptにすることを確かめる（反例が受理されることが、上の限定の根拠である）。

### トレードオフ（3f-b12の節を置き換える）

ownerとpull requestの説明がそのまま引用できるように書く。数は3f-b12と同じで、保存した3f-b10の2系列（上流の統合の前）の同時分布から、27 processを復元抽出で20,000回引いて求めた（`records/probability.json`、b12とbyte単位で同じ）。

Widthで得るもの。

- 許容幅は測定の前に固定してある（ownerの2つ目の決定、2026-11-18）。登録はこの節の文言で取り直した（2026-11-17T23:52:30Z、`variants/width/registration.sha256`）。閾値は3f-b12と同じ数である。
- (a4)と同じ速さの実装は、このbootstrapで2つの系列でそれぞれ99.7%・97.0%受理された。
- Exactなら、同じ実装の受理は約5.6%・0.07%で、9割以上がinconclusive、0.9%・6.8%がslowerだった。

Widthで払うもの。

- 閾値は、認めた値より0.4ポイント上にある。16の合計は1.0100（認めた値1.006）、64の合計は1.0060（1.002）、破棄の費用は1.0150（1.011）。
- (a4)より遅い実装も受理され得る。同じbootstrapで、0.1%遅ければ95.5%・80.2%、0.2%遅ければ72.8%・37.5%、0.3%遅ければ32.0%・4.3%受理された。レビューが作った反例（保存した(a4)を0.2%遅くした系列。16の合計の下側限界1.0061は認めた1.006を上回る）も受理される。
- 認めた値をどれか1つの判定で1%超える実装は、20,000回の抽出で一度も受理されなかった（両系列・4判定とも）。これは保存した分布の下での回数で、実際の測定過程での誤受理率の上限ではない。
- 閾値は観測から求めたUへの受入れ境界で、真の比の上限ではない。真の比が閾値を超える実装も受理され得る。その割合が判定ごとに2.5%以下と言えるのは、processの対数比がi.i.d.正規であるというモデルの下だけで、実際の測定過程や正規でない27標本については証明していない。

どちらの案でも判定しないもの（3f-b12のまま）。nativeの実時間（usableのnative-jsonlとcriterion全体）、新しいprocessでのwire経路、Expression・Branch・TemporalFormulaの破棄経路、usableとcriterion（記録だけ）。

受入れが言えるのは、in-processの4つの判定で観測から求めたUがすべて閾値以下だった、ということだけである。

### 登録

`scripts/register.py`で`variants/width/`と`variants/exact/`の`registration.json`と`registration.sha256`を取り直した（2026-11-17T23:52:30Z、UTC）。`coverage`は上のP1の3点と変えないものを述べ、`inputChecks`に種類の記録、完了taskの出力、task idを足し、`run`に状態機械と`judgement`を書いた。閾値、`judged`、`alpha`、`statistic`、`blocks`、`arms`、`probabilities`は3f-b12と同じである。`begin`が照合するのは`variants/width/registration.sha256`で、そのsha256は`e38c79c15bd4b2d6f8cb84dda5d92fad8f51f443de8543e8b5239415d16be2ee`、`variants/width/registration.json`のsha256は`782d9459a33e6ca606aa7094f7999dff6c86be54bc7080e47823563b5cd4195c`である。登録ファイルのうち`scripts/accept.py`・`scripts/tdist.py`・`scripts/inprocess.mjs`・`records/probability.json`・`records/decision.json`はb12とbyte単位で同じである。

### 自己検査と負の対照

`scripts/accept_selftest.py`は189/189で通った（`records/accept-selftest.txt`、sha256は`records/selftest.sha256`）。importしてもmainは走らない（3f-b12の自己検査はimportでmainが走り、レビューで指定外の場所に一時directoryを作った）。3f-b12から引き継いだ統計・計算の入力検査・converter・C1の検査はそのまま通り、runの状態、C2-A、C2-B、task id、driver、P1の検査を足した。driverの検査はharnessを代役に替え、Pythonのbytecodeを書かせない。benchmarkとbuildは走らせていない。

負の対照（`scripts/negative_controls.py`、`records/negative-controls/`）。登録したファイルを変えずにメモリ上で1か所ずつ戻し、同じ自己検査を走らせた。

| メモリ上で戻したもの                                                                                       | 結果    | 落ちた検査                                                                                                                                           |
| ---------------------------------------------------------------------------------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| 3f-b11の入力検査（`block_differences`）                                                                    | 177/189 | C2の計算の12件                                                                                                                                       |
| 3f-b10のconverter（`proc-*.json`をすべて読む）                                                             | 174/189 | converterの12件と、種類による拒否の3件（stray directory、symlinkとpipe）                                                                             |
| Widthの閾値をExactの値にする                                                                               | 166/189 | 判定がacceptになることを前提にする23件（C1の7件、P1の反例2件、lifecycle・C2-A・C2-B・task idの検査）                                                 |
| taskの開始を記録しない（3f-b12の`admit`と同じく、start・complete・failで状態を変えず、完了の照合もしない） | 163/189 | C2-Aの12件（moduleでのレビューの試行列、開始と完了の間のcrash、二重開始、開始前の出力など）、lifecycleの9件、converterの完了照合4件、全lifecycle 1件 |
| snapshotを一時の場所を経ずに`sealed/`へ直接書く                                                            | 184/189 | 中断したsealの4件と、sealing中に足した観測の1件                                                                                                      |
| `report.json`を直接書く                                                                                    | 187/189 | 書込み途中で中断したreportの2件                                                                                                                      |
| listingがすべての項目をファイルとしてhashする（3f-b12と同じ）                                              | 183/189 | 種類の記録と拒否の6件（stray directoryはIsADirectoryError、pipeは5秒の制限で打ち切り）                                                               |
| usableのtask idの検査を外す                                                                                | 185/189 | レビューの複製2件、前半と後半の入替え1件、runを通した複製1件（完了の照合が別の理由で拒否するので、理由の検査で落ちる）                               |

driverの検査は本物の`measurement_run.py`を別processで呼ぶので、メモリ上の対照の影響を受けない。そのため、taskの開始を記録しない対照では、driverでのレビューの試行列ではなく、同じ列をmoduleの呼出しで確かめる検査が落ちる。

### dry run（受入れの証拠ではない）

- 保存した3f-b10の系列での判定の再計算（`scripts/dryrun.py`）は、`records/dryrun.json`・`records/dryrun.txt`ともb12とbyte単位で同じだった。確率（`scripts/probability.py`）もb12とbyte単位で同じだった。
- driverの全段を、保存した2系列目のprocessファイルと代役のharnessで走らせた（`scripts/lifecycle_dryrun.py`、`records/lifecycle-dryrun.txt`）。begin、counters（36回、代役の`spin`）、in-process 27、封印（accept）、usable 1,032、criterion 240、報告がすべて終了コード0で進み、runはreportedになった。遷移はopen → sealing → sealed → reporting → reported、報告の`judgement`は自分のrunのaccept、変換の拒否はなかった。state.jsonは最後に270,988 byteだった。

### 時間の見積り

各taskの前後に`start`と`complete`の2回の呼出しが入る。どちらも計測区間の外（harnessとharnessの間）で、計測値には入らない。dry runでは、代役のharnessを含むtaskあたりの時間が0.199〜0.211秒、代役だけで0.043秒だったので、2回の呼出しはtaskあたり約0.16秒である。約1,300 taskで約3.5分増え、全体の見積りは約2時間20分から約2時間24分になる。封印は開始から約16分のまま（in-processの28回分で約5秒）。

### 3f-b12・3f-b11・3f-b10から変えないもの

- ownerの2つの決定（選択肢Aで認めた1.006・1.002・1.011、Widthで承認した閾値1.0100・1.0060・1.0150）と、Exactを選ばれなかった案として残すこと。
- 統計（鏡像blockとprocessを単位とする対数比、片側97.5%の上下限、交わりで受理、Berger 1982）、判定4件（in-processのsize 16・64のbefore比とnorm比）、記録だけのfamily、blockとprocessの数（27・129・40）、arm（統合した木から作るimpl・before・norm・placebo）、判定の種類、inconclusiveをGreenにしないこと、記録familyの拒否が判定を変えないこと、入力が拒否されたrunを`refused`として封印すること。
- 計算の入力検査（1つのrun id、計画した0..n−1、(path, block, arm)ごとの値の数）とin-processのconverterの内容検査。
- 上流main（d147550c）を統合してから測ること、順序（build → 命令数・確保回数 → in-process → 封印 → usable → criterion → 報告）、破棄の設計、テスト段の21件、凍結ファイルの数、stackの上限、番号（統合で0048へ移る）、b8の残課題。
- bootstrapの表、「20,000回で受理0回」、Monte Carloの上限約0.00015、dry runの判定。

### 3f-b12から変えたもの

- runの状態（`run.json`と3つのファイルの有無）を、`state.json`の8状態に替えた。`admit`は`start`・`complete`・`fail`に替わり、`verify`は各出来事の前の不変条件の検査と`status`に替わった。
- 封印は書込み1回ずつのファイル（`inprocess-observations.jsonl`・`verdict.json`・`verdict.sha256`）から、`sealed/`の4ファイルを1回のrenameで公開する形に替えた。報告もrenameで公開する。
- 封印の後で凍結した入力が変わったときの扱いを、次のadmitの拒否（戻せば続けられる）からfailedに替えた。凍結した記録と違う中身のまま続けられる状態を残さないため。
- criterionの`ledger.jsonl`とbaseline名`{arm}-{番号}`を、state.jsonの完了記録とtask idのbaseline名に替えた。usableの出力に`taskId`を足した。
- `begin`は同じrootに終わっていないrunがあれば拒否し、以前のrunを記録する。報告に`judgement`・`previousRuns`・familyごとの完了task数を足した。
- 本文1354行の「Uの意味」と1390行の「通り得る真の遅さの上限は各閾値」、1356行の「ほぼ確実に受理される」、registrationの`coverage`を、上のP1とトレードオフの文言に替えた。
- 自己検査、確率、dry runのscriptはmainをimportで走らせない形にし、出力先をこのdirectoryにした。

### 残すもの

- 受入れが言えるのはin-processの4判定で観測したUが閾値以下だったことだけで、真の比の上限も、実際の測定過程での誤受理率の上限も述べない。nativeの実時間、新しいprocessでのwire経路、Expression・Branch・TemporalFormulaの破棄経路は判定せず、usableとcriterionは記録だけである。
- 承認した幅の内側では、(a4)より約0.2〜0.3%遅い実装も通り得る（保存系列のbootstrap）。
- state.jsonとlockは1つのrunを1つのdriverが順に進めることを前提にする。同じrunへ同時に出来事を送ると、lockが1つずつにし、走っているtaskのある状態ではほかの出来事がrunをfailedにする。
- harnessの出力の中身（run id、task id、標本）を書き換えて偽造することは防がない。防ぐのは、失敗や中断の後の測り直し、計画名への複製や置換え、書込みの途中の状態である。
- usableとin-processの差約1.5%を因果で分けていないこと、統合や系列の違いで比が0.2%以上動けば(a4)と同じ実装もinconclusiveかslowerになり得ること（その場合は再測定せずownerへ返す）、`Value::normalize`以外の深い値の再帰がb8の残課題であることは3f-b12のまま。

## step 3f-b14 design: 封印の判定の永続化と、rootの開始の排他

改訂: 2026-11-18 初版（前提レビュー前の草案）。3f-b13の前提レビューはC2-AとP1を閉じ、NO-GO（C2-B・L1）を返した。3f-b13からの変更は次の2点と、その検査だけである。設計、統計、判定4件、閾値、n、arm、測定の手順、テスト段の21件は変えていない。測定runの道具は一度だけ走らせるsession用で、commitしない。道具・自己検査・記録は`<scratch>/step3f-b14-design/`（以下、記録の場所はここからの相対path）に置いた。3f-b13の記録は`b13:`を付けて示す。

- **C2-B（公開した判定が、sealingからのfailで履歴から落ちる）**。封印の最初の出来事（open → sealing）が、`inprocess/`のlistingの凍結と同じstate.jsonの書込みで、公開する判定とsnapshotのdigestを持つようにした。snapshotはその後で`sealed.partial/`に作って公開する。公開は、凍結したlistingから作り直したsnapshotの判定とdigestが永続化した値と等しいときだけ行い、違えばfailedにする。sealing以後のfailedは、公開の前後を問わず永続化した判定を持ち、`previousRuns`と`judgement`がそれを引き継ぐ。3f-b13は判定をsnapshotの公開の後（sealed）にだけstate.jsonへ書いたので、公開済みでsealedへ進む前のrunをfailにすると判定がnullになり、次のrunの判定が採られた。
- **L1（同じrootのbeginが排他でない、同秒のrunの順序がランダムなsuffixで決まる）**。`begin`は登録の照合の後でrunsのrootのlock（root内の`root.lock`への`flock`）を取り、同じ排他区間で、以前のrunがすべて終端であることの確認、以前のrunの履歴の読取り、開始順の番号（`began.sequence`）の付与、新しいrunの最終名での公開（renameとdirectoryのfsync）を行う。後着のbeginは、先行のrunが終端になるまで拒否される。`previousRuns`は開始順の番号で並べ、`judgement`はその順で最初にaccept・inconclusive・slowerを持つrunの判定とした。run名の辞書順は使わない。以前のrunの番号がちょうど1..nでない（重複、欠け）ときは`begin`が拒否する。lockはprocessが終われば（SIGKILLでも）OSが外す。

### 状態表の差

状態の種類と出来事の種類は増えていない。3f-b13の状態表からの差は次のとおりである。

| 状態・出来事                                      | 3f-b13                                                                                          | 3f-b14                                                                                                                                                             |
| ------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| open・seal                                        | sealing（listingと封印時刻を凍結）                                                              | sealing（listing、封印時刻、判定、snapshotのdigestを凍結。snapshotはまだ作らない）                                                                                 |
| sealing（state.jsonが持つもの、常に成り立つこと） | listingと封印時刻。`sealed/`は無いか、凍結したlistingから決定的に作るsnapshotとbyte単位で等しい | + 判定とsnapshotのdigest。`sealed/`は無いか、そのdigestのsnapshotとbyte単位で等しい                                                                                |
| sealing・seal（公開）                             | 凍結した中身が変わらず公開した、または公開済みが同じ → sealed                                   | 作り直したsnapshotの判定とdigestが永続化した値と等しく、公開した、または公開済みが同じ → sealed。作り直したものが永続化した値と違う、または公開済みが違う → failed |
| failed                                            | 封印済みの判定は、元の状態がsealed・recording・reportingのときだけ                              | 元の状態がsealing・sealed・recording・reportingのとき、永続化した判定                                                                                              |
| begin（runなし）                                  | 同じrootの以前のrunがすべてreportedかfailed                                                     | 同じ + 上の排他区間、開始順の番号、以前のrunの番号が1..n                                                                                                           |

### 不変条件を検査する場所

3f-b13の本文は、すべての不変条件を出来事の前に検査すると読めた（前提レビューの非ブロッキング注記）。検査する場所は次のとおりである。

- 出来事の前。`apply`がreportedとfailed以外のrunのstate.jsonを読んだ直後に`violations`で調べる。登録ファイルのdigestがbeginの時と同じこと（終端でないすべての状態）、`inprocess/`のlistingが凍結したlistingと等しいこと（sealing以後）、公開済みsnapshotのdigestが封印したdigestと等しいこと（sealed以後）。崩れていれば出来事を`violation`に替えてfailedにする。`status`は同じ検査をfailed以外のrunについて書込みなしに行い、reportedでは`report.json`のdigestも調べる（reportedは読む以外を拒否するので、これは読取りだけである）。
- 公開を再開するときだけ（sealingでのseal、reportingでのreport）。既存の`sealed/`が作り直したsnapshotとbyte単位で等しいこと、作り直したsnapshotの判定とdigestが永続化した値と等しいこと、既存の`report.json`が作り直した報告とbyte単位で等しいこと。start、complete、fail、statusの前には調べない。この間に`sealed/`や`report.json`を変えても、statusのviolationsは空のままで、startは通常の拒否になり、sealかreportを再開したときにfailedになる。
- 調べない（構成で保つ）。完了taskが計画の先頭からの連続であること、走っているtaskが1つ以下であること（start・completeの組合せで保たれる）。

### 自己検査と負の対照

`scripts/accept_selftest.py`は223/223で通った（`records/accept-selftest.txt`、sha256は`records/selftest.sha256`）。3f-b13の189件のうち、期待値を変えたのは1件、読み方を変えたのは2件で、34件を足した。importしてもmainは走らない（3f-b13のまま）。benchmark、build、worktreeのテスト、TLC、mutationは走らせていない。

- 期待値を変えた1件: 「sealingの間に足した観測は入らず、runはfailedになる」。判定は、3f-b13ではnull、3f-b14では封印の時に永続化したaccept。レビューの修正が求めた結果である。
- 読み方を変えた2件: rootの一覧に`root.lock`が入るため、begin直後の一覧は`root.lock`を含む一覧と比べ、失敗したrunのidはdirectoryだけの一覧から取る。
- C2-Bの9件。レビューのpublishedThenFail（slower（比1.03）のsnapshotを公開し、state.jsonをsealedへ書く直前で中断）で、state.jsonがsealingのまま公開した判定とsnapshotのdigestをすでに持つこと、failしたrunがslowerを持つこと、次のrun（acceptの入力）の`previousRuns`にslowerが入り、報告の`judgement`がslowerのままであること。公開の前の中断（`sealed/`も`sealed.partial/`も無い）でも同じであること、次のsealが永続化したdigestのsnapshotを公開すること、永続化したdigestを書き換えると公開せずfailedになり判定が残ること、公開済みの`sealed/`を変えてsealを再開するとfailedになり判定が残ること。
- L1の13件。レビューのconcurrentBegin（2つのbeginが履歴を読み終えるまで待ち合わせる）と、同秒・suffix `ffffffffffff`と`000000000000`の2つのbegin（同秒の再現）の2つの場合それぞれで、公開されるrunは1つ、後のbeginは`is open`で拒否、公開されたrunの番号は1・以前のrunなし・partialの残りなし、拒否の後にlockは外れていてrunを終えた後のbeginが番号2で公開されること（各3件）。レビューの同秒の順序（A: ffffffffffff、slower（比1.02）、B: 000000000000、accept、C）で、名前順ではBがAより前になるが、Cの`previousRuns`は開始順のA、B、Bの報告もCの報告も`judgement`はAのslower（3f-b13のCはBのaccept）であること。番号が重複する、または欠けるとbeginが拒否すること。別processがlockを持つ間はbeginが待ち、そのprocessをSIGKILLで殺すとbeginが公開されること。
- 登録の12件。`registration.json`のうち`fixedAt`・`supersedes`・`run`以外がb13と等しいこと（width・exactの各1件）、`scripts/accept.py`・`tdist.py`・`convert.py`・`inprocess.mjs`・`usable-run.mjs`・`measure-run.sh`・`probability.py`・`dryrun.py`と`records/decision.json`・`records/probability.json`がb13とbyte単位で同じであること。

負の対照（`scripts/negative_controls.py`、`records/negative-controls/`）。3f-b13の8つを同じ223件で走らせ直し、修正ごとに1つ、順序の修正にもう1つを足した。

| メモリ上で戻したもの                                                                    | 結果    | 落ちた検査                                                                                                                                                           |
| --------------------------------------------------------------------------------------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 3f-b11の入力検査                                                                        | 211/223 | C2の計算の12件（3f-b13と同じ）                                                                                                                                       |
| 3f-b10のconverter                                                                       | 208/223 | 15件（同じ）                                                                                                                                                         |
| Widthの閾値をExactの値にする                                                            | 199/223 | 24件（3f-b13は23件。1件はsealingの間に足した観測の検査がacceptを前提にするため）                                                                                     |
| taskの開始を記録しない                                                                  | 195/223 | 28件（3f-b13は26件。2件は同時beginの後のbeginの検査）                                                                                                                |
| snapshotを直接書く                                                                      | 218/223 | 5件（同じ）                                                                                                                                                          |
| `report.json`を直接書く                                                                 | 221/223 | 2件（同じ）                                                                                                                                                          |
| listingがすべての項目をhashする                                                         | 217/223 | 6件（同じ）                                                                                                                                                          |
| usableのtask idの検査を外す                                                             | 219/223 | 4件（同じ）                                                                                                                                                          |
| **`no-persisted-verdict`**（sealingの永続化した判定を読まない＝3f-b13のfailedと同じ）   | 215/223 | 8件。C2-Bの上の9件のうち7件（failedが判定を持たず、次のrunの`judgement`がaccept）と、期待値を変えた1件                                                               |
| **`no-root-lock`**（`begin`がlockを取らない）                                           | 217/223 | 6件。同時beginの2つの場合で、両方のrunが公開されて拒否が無いこと、その後のbeginが番号2で公開されないこと（各2件）、別processが持つlockを待たない1件、rootの一覧の1件 |
| **`lexical-order`**（`previous_runs`をrun名の辞書順にし、番号を調べない＝3f-b13と同じ） | 219/223 | 4件。Cの`previousRuns`の順序、`judgement`がBのaccept、番号の重複と欠けを拒否しない2件                                                                                |

### 登録

`scripts/register.py`で`variants/width/`と`variants/exact/`の`registration.json`と`registration.sha256`を取り直した（2026-11-18T00:24:31Z、UTC）。`registration.json`の変更は`fixedAt`、`supersedes`（3f-b13を指す）、`run`（begin・封印・`judgement`の規則）の3項目だけで、閾値、`judged`、`alpha`、`statistic`、`blocks`、`arms`、`probabilities`、`coverage`は3f-b13と同じである（自己検査で確認）。

登録のdigestは変わった。登録に列挙した12ファイルのうち2つを直したためで、避けられない。`scripts/measurement_run.py`は`7986d758…`から`cdb9bf83…`、`scripts/accept_selftest.py`は`35f2a508…`から`12e1f687…`になった。残りの10ファイルはb13とbyte単位で同じである。`begin`が照合する`variants/width/registration.sha256`のsha256は`e38c79c15bd4b2d6f8cb84dda5d92fad8f51f443de8543e8b5239415d16be2ee`から`0306c1ac3bac2d9e63151d05406d10a83fb2e852f59b39a1b7d0a35cdbfd38fe`、`variants/width/registration.json`のsha256は`782d9459a33e6ca606aa7094f7999dff6c86be54bc7080e47823563b5cd4195c`から`21b290d1c22f9fd17744829fec1ee0528c3d44efd8dac7abfb61e6f08d11aff6`になった。

### dry run（受入れの証拠ではない）

driverの全段を、保存した2系列目のprocessファイルと代役のharnessで走らせ直した（`scripts/lifecycle_dryrun.py`、`records/lifecycle-dryrun.txt`）。begin、counters、in-process 27、封印、usable 1,032、criterion 240、報告がすべて終了コード0で進み、runはreportedになった。遷移はopen → sealing → sealed → reporting → reported、報告の`judgement`は自分のrunのaccept、変換の拒否はなかった。taskあたりの時間は0.208〜0.253秒（3f-b13は0.199〜0.211秒）、state.jsonは271,005 byteだった。統計とdry run・確率のscriptはb13とbyte単位で同じなので、`records/dryrun.json`・`records/dryrun.txt`・`records/probability.json`はb13のものを引き継いだ（走らせ直していない）。

### 変えないもの、残すもの

- ownerの2つの決定、統計、判定4件、閾値、n、arm、テスト段の21件、測定の順序は3f-b13のままである。
- `begin`のlockは同じrootの開始だけを排他にする。state.jsonと個別runのlockは、1つのrunを1つのdriverが順に進めることを前提にする（3f-b13のまま）。
- 3f-b13の道具で作ったrunは`began.sequence`を持たず、以前のrunとして数えるとbeginが拒否される。測定のrootは新しく作る。
- Exact登録に残るWidth用反例の文言（前提レビューの非ブロッキング注記）には触れていない。採用したWidthの判定には影響しない。

## step 3f-b15: upstream integration and test stage

2026-11-18。上流mainの`d147550c`（#61、モジュール解決と設定置換）へ、このbranchの作業（3f-b8までの実装と3f-b9〜b14の設計）を統合した記録と、実装の前に置くテスト段である。実装は変えていない。commitとstageは行っていない。

### 統合の結果

- 競合は10件。`crates/idealis-core/src/lib.rs`、`crates/idealis-core/tests/tla.rs`、`crates/rtla-compiler/README.md`、`crates/rtla-compiler/SUPPORT.md`、`crates/rtla-compiler/src/lib.rs`、`docs/rust-frontend.md`、`docs/rust-safety.md`、`evals/runs/ledger.jsonl`、`src/contracts.ts`、`src/tla-source.ts`である。いずれも両側を残した。上流の`modules`・source bundle・linkの経路と、この枝の`irVersion`・v2の経路を併せ、`compile_tla_bundle`が`CompileOptions`を受けて`compile_with_options`か`compile_modules_with_options`を呼び、`TlaSourceInput`が`TlaSourceOptions`を継ぎ、ledgerは上流の0047の行とこの枝の0048の行を並べた。
- 番号。上流が0047を`module-linking`に使ったので、この計画・記録・run（`evals/plans/0048-operator-sharing.md`、この記録、`evals/runs/0048-operator-sharing-*`、ledgerのrunId）を0047から0048へ移した。上流の0047の文書とrunは変えていない。
- 統合した木で1件ずつ順に走らせた結果。rtlcの`cargo test --all-targets --locked`は207件（3f-b8と同じ）、idealis-coreは44件（3f-b8は43件、上流の`tests/tla.rs`が1件加える）、rtla-compilerは103件（3f-b8は97件、上流の`tests/modules.rs`が6件加える）、rtlaは12件、すべて通った。rtlc・idealis-core・rtla-compilerのclippy `-D warnings`とfmtも通った。Nodeはanalysis parity 4236/4236、operator-sharing 970/970、frame decoder 153/153、v1 byte identityとbaseline 257/257、TLC regression 1127/1127、`just safety-test` 1183/1183、`just test` 77、`just typecheck`が通った。`just portable-build`、`just portable-test`、`just browser`、`just usable-test`、`just criterion-test`、`just cross-test`、`just async-test`、`just go-test`も通った。`just support-check`は既知のstaleでexit 3のままで、`just support-sync`はAPI key不在のskipである。どちらも同期や鮮度検査の成功には数えない。
- `just check`は統合の時点でerrorが1件で、この記録の表のbiome整形だった。残りは警告3件とinfo 2件である。この段で表の整形だけを直し、`just check`は通る。
- 凍結リストの変更。68件のうち1件、`evals/fixtures/tlc-operator-sharing/analysis-boundary-manifest.json`のdigestを取り直した。理由は、上流のparser変更で`F!1`に対するRust parserの記録が`Unexpected token after expression`から`Expected identifier`へ変わり、manifestのその1行を現状へ合わせたためである。数は68のままで、残る67件のbyteは変わらない。第1270行は「数は70のまま」と`crates/idealis-core/tests/tla.rs`を例に挙げていたが、凍結リストは68件で`tla.rs`は凍結リストにない。この段の後は71件になるので（下の追補）、第1270行を68件・上の1件・テスト段の後で71件、へ直した。

### 決定2: v2は#61のモジュール連結と設定の上書きを対応しない（orchestratorの既定、ownerの判断待ち、選択肢A）

safety IR v2は#61の機能をすべて対応しない。v2の要求でモジュールの連結が要るもの（ユーザーモジュールのEXTENDSまたはINSTANCE、置換）、または設定の上書きがあるもの（operator置換、定数表、定義の上書き）は、先に1つの明示した診断で拒否する。診断は`Unsupported TLA input: safety IR v2 does not support module linking or configuration operator substitution`である。v1と既定の経路は変えない。ownerの判断が出るまで、この既定で進める。

根拠。`crates/rtla-compiler/tests/modules-support.json`の37件のうち、23件はv1でcompileでき、IR v2では`Unsupported TLA input: recursive operator definitions`で拒否される。残る14件は期待値が`unsupported`で、v1とv2が同じ連結の診断で拒否する。23件の拒否は偶然である。linkは別名の定義へ対象の位置（`location`）をそのまま写し、v2は定義を`definition.location.start`で識別するので、別名と対象が同じ定義になり、循環検査が再帰と判定する。連結を持たない入力でも、`compile_modules_with_options`が連結器を通す経路（追加module集合を渡す要求、空の集合を含む）では、別名ができるので同じ拒否が起きる。3f-b16の追補でこの経路を直した。

### テスト段

テストは実装の前に置き、期待値は設計と決定から取った。下書き（`<scratch>/step3f-b10-design/drafts/`の`drop_budget.rs`と`value_normalize_errors.rs`）は、rustfmtだけを当てて`crates/rtlc/tests/`へ置いた。下書きとの差は折り返しと空行だけである（drop_budgetが4か所、value_normalize_errorsが1か所）。

| テスト                                                        | 状態（統合した木、3f-b8の実装） | 失敗の形                                                                                                              |
| ------------------------------------------------------------- | ------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `crates/rtlc/tests/drop_budget.rs`（10件）                    | 赤8件・緑2件                    | 赤は8件とも`tests/drop_budget.rs:120`の`allocations during the drop for case N`で`left: 1`・`right: 0`。abortはしない |
| `crates/rtlc/tests/value_normalize_errors.rs`（11件）         | 緑11件                          | 防護で、まだ壊れていないため赤にならない（3f-b10の記録どおり）                                                        |
| `crates/rtla-compiler/tests/operator_sharing.rs`の拒否（5件） | 赤3件・緑2件                    | 赤の3件は`left`が`Unsupported TLA input: recursive operator definitions`、`right`が決定2の診断                        |

drop_budgetで最初に落ちたcaseは、`shallow_quantifier_with_a_non_leaf_value_domain_drops_without_allocation`・`every_expression_rule_over_a_shallow_non_leaf_child_drops_without_allocation`・`every_other_expression_owner_edge_over_a_shallow_non_leaf_child_drops_without_allocation`・`every_formula_owner_edge_over_a_shallow_non_leaf_child_drops_without_allocation`・`every_value_container_over_a_non_flat_child_drops_without_allocation`・`ordinary_values_and_programs_drop_without_allocation`がcase 0、`value_chain_up_to_the_depth_budget_drops_without_allocation`が34、`expression_chain_up_to_the_depth_budget_drops_without_allocation`が33である。緑の2件は予算の上側の境界（`value_chain_one_level_past_the_depth_budget_allocates_one_worklist`は35段、`expression_chain_one_level_past_the_depth_budget_allocates_one_worklist`は34段で1回）で、3f-b8がどの鎖でもworklistを1回確保することを固定する。3f-b10の記録と同じ赤と緑の内訳である。

拒否のテストは、IR versionの挙動を持つ既存の`operator_sharing.rs`（`IrVersion::try_from`とv1・v2のcompileを検査する）へ足した。`modules-support.json`から2件を取り、IR v2で決定2の診断を完全一致で要求し、同じ入力がv1でcompileでき`version`が1であることを別のテストで要求する。診断は連結と置換で1つの文にしてあるので、どちらの機能が先に検出されてもテストは変わらない。公開の入口はidealis-coreの`compile_tla_bundle`と同じ分け方で、bundleのある入力は`compile_modules_with_options`、ない入力は`compile_with_options`である。

| 入力（`modules-support.json`の`name`） | 内容                                                      | v1                                                   | v2                                                  |
| -------------------------------------- | --------------------------------------------------------- | ---------------------------------------------------- | --------------------------------------------------- |
| `instance-twice`                       | 同じmoduleを2つのINSTANCEとWITH置換で使う（bundleあり）   | 緑 `v1_compiles_instances_with_substitutions`        | 赤 `v2_refuses_instances_with_substitutions`        |
| `constant-operator-override`           | 設定の`CONSTANTS Limit <- Bound Add <- Inc`（bundleなし） | 緑 `v1_compiles_configuration_operator_substitution` | 赤 `v2_refuses_configuration_operator_substitution` |

`operator_sharing.rs`は既存の53件が58件になり、既存の53件は通る。rtlcとrtla-compilerのfmt `--check`とclippy `--all-targets -D warnings`は通る。

凍結リストは71件で、`<scratch>/tests-stage/frozen-tests.sha256`に置き、作業木に対して71/71一致を確かめた。統合後の68件に`crates/rtlc/tests/drop_budget.rs`・`crates/rtlc/tests/value_normalize_errors.rs`・`crates/rtla-compiler/tests/operator_sharing.rs`を足した。`operator_sharing.rs`は今まで凍結リストに入っていなかった既存ファイルで、拒否のテストを置いたので加えた。このfileの既存53件も凍結の対象になる。

### 追補: 凍結71件と、コーパス全体の拒否テスト

- 凍結リストは71件にする。`operator_sharing.rs`を加えて拒否のテストも凍結し、実装段の不変検査が覆う。70件の版（`operator_sharing.rs`を除く）は`frozen-tests.70.sha256`として残す。上の「凍結70ファイル」を述べた3f-b10の節の70は、b10のテスト2ファイルを足した時点の数で、実装段が変えない凍結ファイルはこの段の後で71件である。第1230行・第1270行・第1310行・第1474行の凍結ファイルの数を70から71へ直した。
- 決定2の範囲は#61の機能すべてである。モジュール連結（ユーザーモジュールのEXTENDSまたはINSTANCE、置換）と設定の上書き（operator置換、定数表、定義の上書き）が要るv2の要求は、決定2の1つの診断で拒否する。
- 2行だけを固定するのをやめ、`modules-support.json`の37件すべてを表にした拒否のテスト`v2_refuses_every_module_corpus_case_that_v1_compiles_and_repeats_every_v1_refusal`を`operator_sharing.rs`へ足した。各caseをv1とv2でidealis-coreと同じ入口（bundleあり`compile_modules_with_options`、なし`compile_with_options`）からcompileし、v1がcompileできるならv2は決定2の診断と完全一致、v1が拒否するならv2はv1と完全に同じmessageで拒否することを要求する。どちらの分岐も0件でないことも要求する。先の4件（2行のv1・v2）は残す。
- 赤。現在の木では、37件のうち23件がv1でcompileでき、v2は`Unsupported TLA input: recursive operator definitions`を返す。残る14件はv1とv2が同じmessageで拒否し、このテストでも合う。assertionは最初に違ったcaseの名前を出す。現在の失敗は`assertion left == right failed: extends-counter`、`left: Some("Unsupported TLA input: recursive operator definitions")`、`right: Some("Unsupported TLA input: safety IR v2 does not support module linking or configuration operator substitution")`である（`tests/operator_sharing.rs:647`）。23件の内訳は、一時のprobeで37件を数えて確かめた（v1がcompileしv2が再帰で拒否23、v1とv2が同じ拒否14、その他0）。probeは作業木に残していない。

## step 3f-b16: implementation

2026-11-18。凍結71件を変えず、3f-b10が採用した(a4)の有界破棄と、決定2の拒否を実装した。ベンチマークは実行していない（受入れ計算の測定は別の段）。commit・stage・stash・reset・checkoutは行っていない。凍結71件のsha256は開始時・終了時とも71/71一致した。

### 適用したもの

- 有界破棄（a4）。3f-b10の試作`step3f-b10-design/copies/a4`の`crates/rtlc/src/bounded_drop.rs`と`crates/rtlc/src/value.rs`で、b8の同じ2ファイルを置き換えた。統合した木の`crates/rtlc`は、この2ファイルと凍結テストを除いて(a4)の複製とbyte単位で一致するので、試作はそのまま当たる。設計の3f-b9「採用」と3f-b10「採用する設計」のとおりで、`DEPTH_BUDGET`は32、すべての`detach_*`が`budget`引数を受け、`dispose_*`が葉（`Value`はflat）ならその場で破棄、予算0ならworklistへ積み、それ以外は`budget − 1`で再帰する。`release`はpopしたnodeを`DEPTH_BUDGET`で切り離す。`Drop for Value`は`#[inline]`のgateで、直接の子がすべてflat（高さ2以下）ならdrop glueに任せ、そうでなければ`#[inline(never)]`の`release_value`へ入る。`is_leaf_value`・`is_flat_value`・`is_shallow_value`・`dispose_value`は`#[inline(always)]`、`descend_value`は`#[inline(never)]`である。`normalize`は`ModelValue`・`Int`・`Bytes`を参照で検査して`Ok(self)`を返し、`List`は`iter_mut`、`Set`・`Record`・`Map`は`drain(..)`と`extend`でin-placeに改めた。公開型のfield・variant・serde属性・derive・method、README、`check`の結果は変えていない。
- 決定2の拒否。`rtla-compiler`の`compile_with_options`と`compile_modules_with_options`の両方に適用する。診断は`Unsupported TLA input: safety IR v2 does not support module linking or configuration operator substitution`の1つだけである。
- v2の限界の文書化。`crates/rtla-compiler`と`crates/idealis-core`のREADMEとSUPPORTに、v2が連結と設定の上書きを要する入力を拒否すること、v1と既定は従来どおり受理することを書いた。
- `analysis-inventory.json`の`extracted`。`crates/rtla-compiler/src/lib.rs`のdigestと、凍結extractorが拾う文字列（`lowerEvidence/Rust`に決定2の診断文が1件加わる）が変わったので、3f-b8と同じく`extractAnalysisInventory()`だけで`extracted`を再生成した。変わったJSON pathは`/extracted/sourceSha256/crates/rtla-compiler/src/lib.rs`と`/extracted/lowerEvidence/Rust`の2件で、`extracted`以外の10 top-level fieldsは不変だった。再実行で4236/4236になった。

### 決定2の検出規則

要る機能は要求（moduleとconfig）だけから決め、連結後の形（別名の定義が同じ位置を持つこと）は見ない。`link::needs_linking`は次のどれかで真になる。

- rootが標準module（Naturals、Integers、Sequences、FiniteSets、TLC、Apalache、Variants）以外をEXTENDSする。
- rootがINSTANCEを持つ。
- 設定にoperator置換（`<-`）がある。
- 設定の定数代入が、`[Module]`指定を持つ（module別の値・定義の上書き）、引数を持つ（定数表）、rootの定義と同名（定義の上書き）のいずれかである。

後ろの3つは従来`compile_with_options`が連結器へ送る条件で、式を`link::declares_instances_or_overrides`へ移しただけで変えていない。v1の振り分けは同一である。

位置は連結と共通検査の後、loweringの前である。`compile_modules_with_options`が要求から`linking`を求め、`link::resolve`を実行し、`compile_flat(.., linking)`へ渡す。`compile_flat`はv1と共通の検査（EXTENDS、制約付き設定、SPECIFICATIONとINIT・NEXT、状態変数、記号の衝突、定数）をすべて通した後、`Compiler`の構築と`program()`の前で、`linking`かつ`irVersion`がv2なら診断を返す。このため、v1が連結や検査で拒否する入力（欠落・循環・衝突・不正な設定、連結なしのEXTENDS）はv2でもv1と同じmessageで拒否され、v1が受理する連結入力だけが決定2の診断になる。`compile_with_options`は連結を要する入力を従来どおり`compile_modules_with_options`へ送り、同じ規則に入る。それ以外の入力は`linking = false`の`compile_flat`へ進み、従来と同じである。v1と既定の経路は`irVersion`がv2でないので、この診断を返さない。

### 変更したファイル

- `crates/rtlc/src/bounded_drop.rs`、`crates/rtlc/src/value.rs`
- `crates/rtla-compiler/src/lib.rs`、`crates/rtla-compiler/src/link.rs`
- `crates/rtla-compiler/README.md`、`crates/rtla-compiler/SUPPORT.md`
- `crates/idealis-core/README.md`、`crates/idealis-core/SUPPORT.md`
- `evals/fixtures/tlc-operator-sharing/analysis-inventory.json`（`extracted`のみ）
- `evals/iterations/0048-operator-sharing.md`（この節）

テスト、凍結ファイル、manifest、fixture、既存TLCログ、v1 baselineは変えていない。一時のprobeは作業木に残していない。

### 結果

すべて1つずつ順に、memoryの上限を付けた実行器で走らせた。ログは`<scratch>/implement/logs/`にある。

- `cargo test --test drop_budget`（rtlc）10/10、`--test value_normalize_errors` 11/11。
- `cargo test --test operator_sharing`（rtla-compiler）58/58。拒否の4件（`v1_compiles_instances_with_substitutions`・`v1_compiles_configuration_operator_substitution`・`v2_refuses_instances_with_substitutions`・`v2_refuses_configuration_operator_substitution`）とコーパス表の`v2_refuses_every_module_corpus_case_that_v1_compiles_and_repeats_every_v1_refusal`を含む。
- `cargo test --all-targets --locked --no-fail-fast`: rtlc 228（3f-b15の207に新規21）、rtla-compiler 108（103に新規5）、idealis-core 44、rtla 12。すべて通った。
- rtlc・rtla-compiler・idealis-coreのfmt `--check`とclippy `--all-targets -D warnings`。すべて通った。
- Node: analysis parity 4236/4236（再生成の前は4235/4236で、`analysis inventory source digest changed`の1件）、operator-sharing 970/970、frame decoder 153/153、v1 byte identityとbaseline 257/257、TLC regression 1127/1127、`just safety-test` 1183/1183。
- `just portable-build`、`just portable-test`（Rust 392件、実Chromium 2件）、`just check`（Vitest 77、node:test 77）、`just browser`（実Chromium 3件）、`just usable-test` 4/4、`just criterion-test`、`just cross-test` 2/2、`just async-test` 4/4、`just go-test`、`just support-plan`（7 crates）、`git diff --check`。すべて通った。Goは`mise exec`の`go@1.25.14`で走らせ、`GOFLAGS=-buildvcs=false`を付けた。

`just support-sync`はAPI key不在のskip、`just support-check`は既知のstaleでexit 3のままなので、この段では実行せず、どちらも成功には数えない。support記録・digestは手編集していない。

### 残るもの

- 連結を要さない入力を`compile_modules_with_options`へ渡すv2の拒否は、この節の最初の実装では残っていた（一時のprobeで、標準moduleだけをEXTENDSする単一moduleが、空の追加module集合でも無関係な追加moduleを添えても`recursive operator definitions`で拒否されることを確かめた）。下の追補で直した。
- `docs/rust-frontend.md`・`docs/rust-safety.md`・`docs/safety-ir-v2.md`の限界の記述は、下の追補で書いた。
- 受入れ計算の測定は未実施で、(a4)は3f-b10・3f-b11の記録のとおり固定した許容幅の判定を要する。

### 追補: 連結を要さないv2の入力（テスト、修正、凍結72件）

2026-11-18。上の「残るもの」の1件目を、測定の前に直した。測定する実装を最終のコードにするためである。順序はテスト、修正、凍結、文書、全suiteの再実行。

- テスト。新しいfile`crates/rtla-compiler/tests/operator_sharing_bundle.rs`（3件）を足した。凍結の`operator_sharing.rs`は変えていない。行は、標準moduleだけをEXTENDSする単一moduleの4つ（counter、定義を共有するlexicalなoperator、`finite-support.json`の`model-untyped`（定数代入つき）と`temporal-fair`（SPECIFICATIONとfairness））と、追加moduleの3通り（空、無関係な1つ、一方が他方をEXTENDSする無関係な2つ）の組である。
  - `v2_bundle_route_emits_the_ir_of_the_flat_route_when_no_linking_is_needed`: 12組それぞれで、`compile_modules_with_options`のv2の出力が、同じmoduleとconfigの`compile_with_options`のv2のserialize済みIRと一致し、`version`が2であることを要求する。
  - `v1_bundle_route_still_compiles_every_row_that_needs_no_linking`: 同じ12組が`compile_modules_with_options`のv1でcompileでき、`version`が1であることを要求する（v1の対照）。
  - `v2_bundle_route_repeats_every_v1_refusal_for_inputs_that_need_no_linking`: 連結を要さない入力のうち、v1が連結の検証で拒否する4つ（bundleの重複module、予約名のmodule、未解決の識別子を空のbundleと無関係なmoduleの両方で）で、v2がv1と同じmessageで拒否し、決定2の診断ではないことを要求する。
- 赤。修正前は、1件目が12組すべてで`Unsupported TLA input: recursive operator definitions`を返して落ちた（意図した赤）。2件目と3件目は修正前から通る対照である。3件目は、検証を飛ばす誤った修正を防ぐ。
- 修正。`compile_modules_with_options`は、IR v2かつ`link::needs_linking`が偽のとき、`link::resolve`を従来どおり実行して検証の拒否をv1と同じに保ち、その結果は使わず、rootのmoduleとconfigを`compile_flat`へ渡す。v1と、v2でも連結を要する入力は従来の経路のままである。`compile_with_options`は変えていない。感度として、`resolve`の前に早期に`compile_flat`へ渡す誤った実装（検証を飛ばす）を一時に入れると、3件目が`duplicate bundle module: IR v2 compiled the input`で落ちることを確かめ、元に戻した（`lib.rs`のsha256が一致）。
- 緑。3件とも通った。凍結71件とあわせた全suiteも通った（下）。
- 凍結。`operator_sharing_bundle.rs`を加えて凍結リストを72件にした。71件の版は`frozen-tests.71.sha256`として残し、72件の版を`frozen-tests.sha256`へ置いた。両者の差はこの1行だけで、72件とも作業木で一致した。この追補以降、実装段が変えない凍結ファイルは72件である。
- 文書。`docs/rust-frontend.md`・`docs/rust-safety.md`・`docs/safety-ir-v2.md`にv2の限界を書いた（決定2の診断の文言、v1と省略は従来どおり受理、連結を要さない要求は追加moduleを伴ってもv2で通常どおり変換する）。`crates/rtla-compiler/README.md`・`SUPPORT.md`・`crates/idealis-core/README.md`は、連結を要さない入力が`recursive operator definitions`で拒否されるという直前の記述を、同じIRになるという記述に置き換えた。3f-b15の「連結を持たない入力では起きない」の文は、連結器を通す経路では起きたこと、3f-b16の追補で直したことを加えて直した。
- `analysis-inventory.json`の`extracted`。`lib.rs`が変わったので、同じ手順で再生成した。変わったJSON pathは`/extracted/sourceSha256/crates/rtla-compiler/src/lib.rs`の1件で、`extracted`以外の10 top-level fieldsは不変だった。
- 変更したファイル（追補）。`crates/rtla-compiler/src/lib.rs`、`crates/rtla-compiler/tests/operator_sharing_bundle.rs`（新規）、`crates/rtla-compiler/README.md`、`crates/rtla-compiler/SUPPORT.md`、`crates/idealis-core/README.md`、`docs/rust-frontend.md`、`docs/rust-safety.md`、`docs/safety-ir-v2.md`、`evals/fixtures/tlc-operator-sharing/analysis-inventory.json`（`extracted`のみ）、この記録。
- 再実行した結果（すべて逐次、上限つき。最終のコードで）。`operator_sharing_bundle` 3/3、`operator_sharing` 58/58、`drop_budget` 10/10、`value_normalize_errors` 11/11。`cargo test --all-targets --locked --no-fail-fast`はrtlc 228、rtla-compiler 111、idealis-core 44、rtla 12。rtlc・rtla-compiler・idealis-coreのfmt `--check`とclippy `--all-targets -D warnings`。analysis parity 4236/4236、operator-sharing 970/970、frame decoder 153/153、v1 identity 257/257、TLC regression 1127/1127、`just safety-test` 1183/1183。`just portable-build`、`just portable-test`（Rust 395件、実Chromium 2件）、`just check`（Vitest 77、node:test 77）、`just browser`（実Chromium 3件）、`just usable-test` 4/4、`just criterion-test`、`just cross-test` 2/2、`just async-test` 4/4、`just go-test`、`just support-plan`、`git diff --check`。すべて通った。

## step 3f-b17: measurement

2026-11-18。3f-b14が登録した手順（variant `width`、閾値1.0100・1.0060・1.0150）で、3f-b16の追補を終えた最終の木から4つのarmを作り、実装の測定を1回走らせた。実装、テスト、登録、計算は変えていない。凍結72件のsha256は開始時と終了時のどちらも72/72一致した。commit・stage・stash・reset・checkoutは行っていない。

### 判定

**inconclusive**。4件の判定のうち、`16/wasm-inprocess`のimpl/beforeだけがU = 1.0101266で閾値1.0100を0.013ポイント上回り、受理に届かなかった。同じ判定のL = 1.0062818は閾値を超えないので、slowerでもない。残る3件はU ≤ 閾値である。登録した規則では、判定すべてでU ≤ 閾値ならaccept、どれかでL > 閾値ならslower、それ以外はinconclusiveで、inconclusiveはGreenにしない。受入れはこの測定ではGreenにならない。

| 判定                                   | n   | 比 [L, U]               | 閾値   | 結果         |
| -------------------------------------- | --- | ----------------------- | ------ | ------------ |
| in-process 16、impl/before             | 27  | 1.0082 [1.0063, 1.0101] | 1.0100 | inconclusive |
| in-process 64、impl/before             | 27  | 0.9996 [0.9976, 1.0015] | 1.0060 | not-slower   |
| in-process 16、impl/norm（破棄の費用） | 27  | 1.0103 [1.0089, 1.0117] | 1.0150 | not-slower   |
| in-process 64、impl/norm（破棄の費用） | 27  | 1.0096 [1.0083, 1.0110] | 1.0150 | not-slower   |

statusは判定ごとの分類で、`not-slower`はU ≤ 閾値、`inconclusive`はU > 閾値かつL ≤ 閾値、`slower`はL > 閾値である。L・Uはそれぞれ片側97.5%の下側・上側の限界である。

登録の規則に従い、再測定はしていない。判定は最初に測ったこのin-processだけで決まり、封印の後のrunの結果や失敗は判定を変えない。報告の`judgement`は、同じrootで最初に判定を封印したrunの判定であり、このrunの`inconclusive`である。この判定の先、つまり新しいrunで測り直すか、閾値の扱いをownerへ返すかは、3f-b11の「統合や系列の違いで比が0.2%以上動けば、(a4)と同じ実装もinconclusiveかslowerになり得る。その場合は再測定せずownerへ返す」に従い、orchestratorとownerの判断である。

保存した3f-b10の(a4)の系列（統合の前の木、3f-b11のdry run）との並びを示す。木が違うので同じ量の比較ではなく、判定の根拠にも使っていない。

| 判定                                   | 今回のimpl              | (a4)の系列1             | (a4)の系列2             |
| -------------------------------------- | ----------------------- | ----------------------- | ----------------------- |
| in-process 16、impl/before             | 1.0082 [1.0063, 1.0101] | 1.0058 [1.0041, 1.0075] | 1.0060 [1.0046, 1.0074] |
| in-process 64、impl/before             | 0.9996 [0.9976, 1.0015] | 0.9990 [0.9972, 1.0008] | 1.0019 [1.0002, 1.0036] |
| in-process 16、impl/norm（破棄の費用） | 1.0103 [1.0089, 1.0117] | 1.0090 [1.0080, 1.0101] | 1.0096 [1.0085, 1.0108] |
| in-process 64、impl/norm（破棄の費用） | 1.0096 [1.0083, 1.0110] | 1.0091 [1.0081, 1.0102] | 1.0111 [1.0090, 1.0131] |

16のbefore比は保存した系列より約+0.2%高い（1.0082 / 1.0058 = +0.24%、1.0082 / 1.0060 = +0.22%）。登録した確率表では、(a4)を+0.2%遅くした系列でaccept 0.728（系列1）・0.375（系列2）、inconclusive 0.272・0.625、slower 0.000で、今回のinconclusiveはこの表が示す起こり得る結果の範囲に入る。64のbefore比は保存した系列の幅（0.9990〜1.0019）の内側にあり、2件のnorm比のUは1.0117・1.0110で、閾値1.0150を0.3ポイント以上下回る。

### 記録だけのfamily（判定ではない）

in-processのplaceboとnorm、usable、criterionは3f-b11の登録どおり記録だけで、判定にも受入れにも使わない。usableは配置だけで最大2%、criterionは配置だけで−0.5〜+1.7%動くことが3f-b10・3f-b11で分かっており、同じ理由で認めた大きさの退行を見分けられない。

in-process（記録のみ、n = 27）。

| path              | placebo/before          | norm/before             |
| ----------------- | ----------------------- | ----------------------- |
| 16/wasm-inprocess | 1.0007 [0.9988, 1.0025] | 0.9979 [0.9957, 1.0001] |
| 64/wasm-inprocess | 1.0006 [0.9988, 1.0024] | 0.9900 [0.9877, 0.9924] |

placebo/beforeは両sizeで±0.4%（登録した分解能）の内側で、1と区別できない。配置だけの動きは、登録した分解能を超えていない。

usable（記録のみ、n = 129鏡像block、各armが2回）。

| path            | impl/before             | impl/norm               | placebo/before          | norm/before             |
| --------------- | ----------------------- | ----------------------- | ----------------------- | ----------------------- |
| 16/wasm-wire    | 1.0073 [1.0005, 1.0142] | 1.0055 [0.9985, 1.0126] | 1.0004 [0.9942, 1.0066] | 1.0018 [0.9953, 1.0084] |
| 64/wasm-wire    | 1.0069 [1.0010, 1.0129] | 1.0033 [0.9976, 1.0091] | 1.0013 [0.9953, 1.0073] | 1.0036 [0.9978, 1.0094] |
| 16/native-jsonl | 1.0063 [0.9996, 1.0129] | 1.0138 [1.0065, 1.0212] | 1.0026 [0.9961, 1.0091] | 0.9925 [0.9860, 0.9991] |
| 64/native-jsonl | 1.0036 [0.9970, 1.0102] | 1.0036 [0.9962, 1.0111] | 1.0009 [0.9940, 1.0079] | 0.9999 [0.9926, 1.0072] |
| 16/ts-prepared  | 0.9880 [0.9729, 1.0032] | 0.9950 [0.9814, 1.0087] | 0.9980 [0.9838, 1.0124] | 0.9930 [0.9784, 1.0078] |
| 64/ts-prepared  | 0.9970 [0.9860, 1.0081] | 0.9972 [0.9862, 1.0084] | 1.0012 [0.9907, 1.0117] | 0.9997 [0.9891, 1.0105] |
| 16/ts-wire      | 0.9906 [0.9800, 1.0012] | 0.9892 [0.9792, 0.9994] | 1.0020 [0.9918, 1.0122] | 1.0014 [0.9905, 1.0123] |
| 64/ts-wire      | 0.9980 [0.9902, 1.0058] | 0.9953 [0.9877, 1.0030] | 0.9986 [0.9906, 1.0065] | 1.0027 [0.9957, 1.0097] |

criterion（記録のみ、n = 40鏡像block、各armが2回、before・placebo・impl）。

| path                               | impl/before             | placebo/before          |
| ---------------------------------- | ----------------------- | ----------------------- |
| `membership/function-cube`         | 0.9904 [0.9781, 1.0029] | 0.9858 [0.9720, 0.9998] |
| `membership/parallel-channels`     | 0.9866 [0.9712, 1.0021] | 0.9859 [0.9712, 1.0007] |
| `search/counter-product/1`         | 0.9913 [0.9790, 1.0038] | 0.9916 [0.9766, 1.0068] |
| `search/counter-product/8`         | 0.9911 [0.9829, 0.9993] | 0.9983 [0.9791, 1.0179] |
| `search/function-cube/1`           | 0.9997 [0.9925, 1.0070] | 0.9992 [0.9854, 1.0132] |
| `search/function-cube/8`           | 0.9907 [0.9649, 1.0173] | 0.9960 [0.9691, 1.0237] |
| `search/linear-ring/1`             | 0.9982 [0.9897, 1.0068] | 1.0043 [0.9903, 1.0186] |
| `search/linear-ring/8`             | 0.9945 [0.9858, 1.0033] | 1.0011 [0.9862, 1.0161] |
| `search/parallel-channels/1`       | 1.0023 [0.9734, 1.0322] | 0.9881 [0.9690, 1.0076] |
| `search/parallel-channels/8`       | 1.0038 [0.9757, 1.0327] | 1.0013 [0.9900, 1.0127] |
| `values/except-function`           | 0.9973 [0.9912, 1.0034] | 0.9982 [0.9920, 1.0044] |
| `values/except-record`             | 0.9972 [0.9907, 1.0037] | 0.9939 [0.9887, 0.9991] |
| `values/intern-existing-functions` | 1.0090 [1.0011, 1.0171] | 1.0055 [0.9983, 1.0128] |
| `values/intern-new-functions`      | 1.0063 [0.9972, 1.0155] | 1.0023 [0.9954, 1.0092] |

表の区間は、各比のblockごとの対数比から3f-b10と同じ式で求めた片側97.5%の下限と上限である。

### arm

4つのarmはすべて、3f-b16の追補を終えた最終の木（`crates/idealis-core`・`rtla`・`rtla-compiler`・`rtlc`と`fixtures/portable-values.json`）の複製を作業木の外に作り、複製からbuildした。

- impl: 複製のまま。
- before: implから`crates/rtlc/src/bounded_drop.rs`と`lib.rs`の`mod bounded_drop;`行を外し、`crates/rtlc/src/value.rs`を作業木のHEADのもの（`git show HEAD:crates/rtlc/src/value.rs`、`normalize`は従来で、3f-b10が保存したbeforeの`value.rs`とbyte単位で一致する）へ戻した。
- norm: beforeに、implの`value.rs`（`normalize`のin-place化）だけを当てた。
- placebo: beforeに、implの`bounded_drop.rs`へ3f-b10が定義した変更（`impl Drop`の4つを外し、4つの入口を`#[used]`のstaticに置いた関数pointerで保持する）を当てたものと、`lib.rs`の`mod bounded_drop;`行を足した。3f-b10が保存した(a4)の`bounded_drop.rs`とplaceboの`bounded_drop.rs`の`diff -u`を、implの`bounded_drop.rs`（(a4)と同一）へ`patch`でそのまま当て、結果が保存したplaceboの`bounded_drop.rs`とbyte単位で一致することを確かめた。`lib.rs`はimplと同じで、`value.rs`はbeforeと同じである。

各armのimplとの差分は`crates/rtlc/src/`の`bounded_drop.rs`・`lib.rs`・`value.rs`のうち上の変更に当たるものだけで、ほかのfileは1件も違わない（`diff -ruN`を記録した）。さらに、各armの`crates/rtlc/src`は、3f-b10が保存した同名のarm（implは(a4)）の`crates/rtlc/src`とbyte単位で一致した。

| 差分           | file                                      | 行数 | sha256                                                             |
| -------------- | ----------------------------------------- | ---- | ------------------------------------------------------------------ |
| impl → before  | bounded_drop.rs（削除）、lib.rs、value.rs | 588  | `e4277ea46f0ded3df141b3674d1346d0a7758b51366ef4eb4314678005cad402` |
| impl → norm    | bounded_drop.rs（削除）、lib.rs           | 488  | `9b0d5222bf3d6a26db9604537e0a3cfa1d270c7c8c58c48cae292553bac033f4` |
| impl → placebo | bounded_drop.rs、value.rs                 | 166  | `dee9603188e837378f057b2d95bff93fd60888814a9fcdc59d3cd092924dd75a` |

buildはarmごとに新しい複製で行い、成果物を保存した後でbuild出力を消した。rustc 1.100.0、cargo 1.100.0（3f-b10と同じ）。native binaryは`cargo build --release --locked --offline`、wasmは`cargo build --release --locked --offline --lib --target wasm32-unknown-unknown`（どちらも`crates/idealis-core`）、計測harness（`spin`と`count`、3f-b10のharnessをarmごとの複製へ向けたもの）は`cargo build --release --offline`、criterionのbench実行ファイルは`crates/rtlc`で`cargo bench --bench safety --no-run --locked --offline`である。TypeScriptの`dist`はこの段では作らず、作業木にあるbuild（`src`と`evals/usable`のどのfileより新しい）を全armで共有した。694ファイルの`dist`のsha256の一覧は、runの前後で同じだった。

成果物のsha256（run中とrun後で同じことを再計算で確かめた）。

| arm     | 成果物          | bytes   | sha256                                                             |
| ------- | --------------- | ------- | ------------------------------------------------------------------ |
| before  | native          | 2454640 | `2e77562ddcfb91a950079adc4a4ee4ea51bd8ee268f8e7f9a811f325891a3cbe` |
| before  | wasm            | 2852960 | `91a922b9056ca7d8c9da269ef6536a7e33bd4941613c5b1373374ef5e1f96b8a` |
| before  | harness spin    | 3349272 | `58b8b9793507f0a967e07fe7832d5cd805ce766bb4e095547192293af25d670d` |
| before  | harness count   | 3503224 | `1d4fcf502707ba4b9f4e9c771856bbaf3c427d5833ac0f2d966844ff32672b76` |
| before  | criterion bench | 2805712 | `5e4d4653bdc0f99f203938380cf8c3cb70836d20fa96b65eeb9a71c4592bbdde` |
| placebo | native          | 2471264 | `468c3aa432d3efbcfee219636594ac0cc1cc2c2584c6978be79db9bc6400efcf` |
| placebo | wasm            | 2869935 | `13d021b1c7e78ff04fce69ef24bf2541486b99bfb13c72101e364ebc46b17870` |
| placebo | harness spin    | 3369608 | `856475591972e35f3062a9a4d07c47c9442ce99d3931281867a223a868eaab33` |
| placebo | harness count   | 3523560 | `b9e4adef84f6ec0175e879ab410c3b33b045e2ab451ea774047b5ab9552eb47a` |
| placebo | criterion bench | 2822288 | `4c5aa24a596e68c6c45cdd9f991b30960986dae23a9592de9f624488534b7a86` |
| norm    | native          | 2454640 | `da69099d6e0c539ebbd9fd217889dccd5d39822ca87985b8dcaca0117515471e` |
| norm    | wasm            | 2852125 | `f3f9c7c13e63ccfb73ecbb3777c32e22202340f8963fd78314022b9be3170125` |
| norm    | harness spin    | 3347992 | `1b3c79db21e33c8d6903480d523898520fb747c0dce5f28dd3b240ba242079a1` |
| norm    | harness count   | 3501928 | `ae26a139b4e81198dd51205d9efd752adf8f3ef765ae0a26ef0ec2ee9255b1cf` |
| impl    | native          | 2487712 | `1ff8c87a19c9250f71d9db40490501d9039423427a839c42d3560c5f1dbca939` |
| impl    | wasm            | 2878082 | `b8e9e9b844f0f2e9ae35e6b808351d6e48254d0c88575059d88519c48e9175c8` |
| impl    | harness spin    | 3386120 | `d3263b78259cac64c20756e9e0de6129bb3e956b3f734c0fe1fe55da060e4a6b` |
| impl    | harness count   | 3523560 | `d1e8ff1f1f7b0b3e7a52ea27f2338cd82819792599466ab388f065d08cb47b66` |
| impl    | criterion bench | 2822288 | `b2a964821b448a4787a81a2d3bfd4084af3cc6e88248005004d912a89ac3647c` |

大きさ。beforeに対するplaceboは、wasm +16,975 B（3f-b10は+16,975 B）、native binary +16,624 B（+16,608 B）、criterion bench +16,576 B（+16,576 B）で、codeが残り呼ばれないことと整合する。implはwasm +25,122 B（(a4)は+25,068 B）、normはwasm −835 Bである。

### 命令数と確保回数（診断）

確保回数（`count`、1回の`dispatch`）は、3f-b10の記録とすべての値が同じだった。beforeとplaceboは同一で、normとimplが同一である。

| size | before、placebo          | norm、impl          |
| ---- | ------------------------ | ------------------- |
| 1    | 7,592（bytes 1,106,553） | 7,399（1,087,905）  |
| 16   | 21,977（2,542,391）      | 21,364（2,480,063） |
| 64   | 68,009（7,154,327）      | 66,052（6,952,223） |

命令数は、登録した計数の36回（3 round × 4 arm × size 1・16・64、armの順を回す）の平均である。

| size | placebo/before | norm/before | impl/before | impl/norm |
| ---- | -------------- | ----------- | ----------- | --------- |
| 1    | +0.18%         | −0.74%      | −0.52%      | +0.23%    |
| 16   | +0.18%         | −0.60%      | −0.41%      | +0.19%    |
| 64   | +0.29%         | −1.02%      | −0.43%      | +0.59%    |

placeboはbeforeと確保回数が一致し、命令数の差は+0.18%・+0.18%・+0.29%（3f-b10は−0.47%・+0.09%・+0.20%）で、実行する仕事が変わらないことと整合する。

### runと状態

- run id: `20261118T023152Z-a24dd82cbe84`（開始の通し番号1、`previousRuns`は空）。measurement rootは`<scratch>/measure-final/runs`、artifacts rootは`<scratch>/measure-final/artifacts`である。
- 開始の前に、登録の13ファイル（`registration.sha256`が並べる計算、harness、決定の記録、登録）のsha256を照合し、すべて一致した（`registration.sha256`自体のsha256は`0306c1ac3bac2d9e63151d05406d10a83fb2e852f59b39a1b7d0a35cdbfd38fe`）。決定の記録のsha256は`4c08dd9259c6fa68f4432d69503d7b1a56f7be930a2570a97767184c6b8105a5`。run中とrun後にも、`state.json`が持つ登録のdigestと一致していた。
- 状態の遷移は`begin`（なし → open）、`seal`（open → sealing）、`published`（sealing → sealed）、`report`（sealed → reporting）、`report-published`（reporting → reported）の5つで、終端はreported。taskは計数1・in-process 27・usable 1,032・criterion 240が計画の順にすべて完了し、failedになったtaskも拒否された出来事もない。
- 封印のsnapshotのsha256は`0bf817b25864380b37e453e5d386f2cccabbea555d3f31d87cff0873cd2a3a1d`、`report.json`のsha256は`98e6190ebb3c5862f5d14214fe3e49a7eecd45b2b57fb8260f4cae42086b192c`。

| 段                                 | 開始     | 終了     | 時間          |
| ---------------------------------- | -------- | -------- | ------------- |
| begin                              | 02:31:52 | 02:31:52 | 0秒           |
| 命令数・確保回数（計数）           | 02:32:01 | 02:32:38 | 37秒          |
| in-process（27 process）           | 02:32:51 | 02:38:38 | 5分47秒       |
| 判定の封印（sealed、inconclusive） | 02:38:50 | 02:38:50 | 1秒未満       |
| usable（1,032 run）                | 02:39:01 | 02:48:29 | 9分28秒       |
| criterion（240 run）               | 02:48:37 | 04:46:10 | 1時間57分33秒 |
| report                             | 04:46:29 | 04:46:30 | 1秒           |

時刻は2026-11-18のUTC。判定は02:38:50に封印され、usableとcriterionはその後に測った。他のsessionの負荷がある中で測り（段の開始時のload average: in-process 2.9、usable 2.8、criterion 3.6）、この段の測定は1つずつ順に走らせ、測定中にこの段の別のbenchmarkやbuildは走らせていない。driverは`bash measure-run.sh <段>`で起動した（fileに実行bitが付いていない）。disk空きは、確認した範囲で30 GiBを上回り続けた（確認した最小は約32.2 GiB）。

### 残るもの

- 判定はinconclusiveで、受入れはGreenにならない。次はownerの判断である。この判定を変える再測定は、登録した規則では許さない（新しいrunを始めても、報告の`judgement`はこのrunの判定を指す）。
- 判定したのは4件のin-processだけで、native codeの実時間、新しいprocessでのwire経路、`Value::normalize`以外の深い値の再帰は、3f-b11が書いたとおり判定していない。usableとcriterionの記録は、配置だけで動く大きさ（usableで最大2%、criterionで−0.5〜+1.7%）より小さい差を区別できない。
- run、artifacts、記録は`<scratch>/measure-final/`に残してある（build出力は消した）。この段で変えた作業木のfileはこの節だけである。

## step 3f-b18: review 8 fixes

2026-11-18。第八回最終レビュー（NO-GO、P0なし）の確定指摘P1-1・P2-1・P2-2を直した。P2-3（Jevの検査対象マニフェストが古い）は既知の項目として残す。直すにはmaintainerの`support-sync`が要るので、pull requestで依頼する。この段では`support-sync`も`support-check`も実行していない。以下の`<scratch>`は、作業木の外にあるsessionの作業directoryである。

### 指摘と変更

- P1-1（正規化テストの入力に重複キーがない）。`crates/rtlc/tests/value_normalize_errors.rs:61-70`の`map_reports_a_key_error_before_a_duplicate_key`の入力の末尾に`(int("1"), Value::Null)`を足した。名前と期待する診断`Noncanonical integer`は変えていない。期待値は検査順の仕様（キーの正規化を先に行い、第2要素で拒否する）から取った。入力は`[1, 01, 1]`になり、現行の実装は第2要素で`Noncanonical integer`を返し、生のキーを全件先に走査して重複を報告する実装は第3要素で`Duplicate map key`を返す。本番の`crates/rtlc/src/value.rs`は変えていない。
- P2-1（v2の診断と未使用moduleの同一IRを約束する条件）。`docs/safety-ir-v2.md:5`と同じ条件に揃えた。固定診断は、v1と共通の連結と検査が通った場合に返す。未使用moduleを添えた場合の同一IRは、bundleの検証が通った場合に成り立つ。
  - `docs/rust-safety.md:85`: 固定診断に条件を足した。
  - `crates/idealis-core/README.md:19`: 固定診断と同一IRの両方に条件を足した。
  - `crates/rtla-compiler/README.md:45`: 同一IRに条件を足した（固定診断の条件はすでにある）。
  - 同じ約束を述べる`SUPPORT.md`も揃えた。`crates/idealis-core/SUPPORT.md:7`（固定診断）、`crates/rtla-compiler/SUPPORT.md:15`（同一IR）、`crates/rtla-compiler/SUPPORT.md:47`（固定診断）。
  - 上記以外の文は変えていない。
- P2-2（反復ノートのローカルな絶対path）。このノートの13行にあった`<scratch>`の元のroot以下の絶対pathを、`<scratch>/...`へ置き換えた。artifactの相対path、run ID、digestは残した。レビューは14個と数え、行は12行を挙げたが、作業用の絶対path（scratchのroot、home directory、利用者名）を探す検索は13行に一致した。置換後はこの13行がなくなり、作業木の変更・未追跡の全ファイル（285件）に同じ検索の一致がない。

### 凍結リスト

`crates/rtlc/tests/value_normalize_errors.rs`だけを、許可された1件の変更として凍結リストから取り直した。変更の前に72件すべてが一致することを確かめ、変更の後に新しいリストへ72件すべてが一致することを確かめた。新しいリストは`<scratch>/step3f-b18/frozen-tests.sha256`に置いた。古いリスト`<scratch>/tests-stage/frozen-tests.sha256`は変えていない。2つのリストの差は12行目の1行だけである。

- 変更前: `3eb1f860030e5a33e6f355b333b317ea080c9b22d9b0d3ed53f1060cb163dc18`
- 変更後: `a84fac621208804181a4d92dbc0c8d9486538af8fe14fccd1f6e62778a22648b`

### 感度

作業木の外に`crates/rtlc`の複製（`target`を除く）を作り（`<scratch>/step3f-b18/scratch/rtlc`）、複製の`Value::normalize`のMapの扱いだけを変えた。正規化の前に、生のキーを`serde_json::to_string`した形で全件走査し、重複があれば`Duplicate map key`を返す。複製は専用のtarget directoryで`cargo test --locked`を走らせ、終わったあとtargetを消した。ログは`<scratch>/step3f-b18/logs/`。

- 複製（変更あり）で新しいテスト`map_reports_a_key_error_before_a_duplicate_key`を走らせた（`sens-mutated-new-test-exact`）: 失敗。`left: ("value", "Duplicate map key")`、`right: ("value", "Noncanonical integer")`。
- 同じ複製でファイル全体を走らせた（`sens-mutated-new-test-whole-file`）: 11件のうち9件が成功、2件が失敗。失敗は上のテストと、既存の`map_reports_a_value_error_before_a_duplicate_key`（`Duplicate map key`が返り、期待は`Invalid lowercase hex bytes`）。
- 同じ複製で変更前の入力`[1, 01]`のテストを走らせた（`sens-mutated-old-test-exact`）: 成功。変更前の入力は重複を先に走査する実装を区別できなかった。P1-1の指摘のとおりである。
- 作業木の本物（変更後のテスト）: 11/11成功（`real-value-normalize-errors`）。

### 再実行

すべて1つずつ順に、memoryの上限を付けた実行器で走らせた。ログは`<scratch>/step3f-b18/logs/`にある。

- `cargo test --test value_normalize_errors`（rtlc）11/11、`--test drop_budget` 10/10。
- `cargo test --all-targets --locked --no-fail-fast`: rtlc 228、rtla-compiler 111、idealis-core 44。すべて通った。
- rtlcのfmt `--check`とclippy `--all-targets -D warnings`。どちらも通った。
- `git diff --check d147550c`。通った。
- `just check`（biome check 571ファイル、typecheck、Vitest 77/77、node:test 77/77）。通った。biomeの警告3件・情報2件は、この段で触れていない既存のファイル（`evals/alloy-subset/run.ts`、`evals/smt-subset/benchmark.ts`、`src/backend.ts`、`src/tla-safety.ts`）のものである。
- ノートに対する`biome check`。通った。

## step 3f-b19: review 9 fix

2026-11-18。第九回最終レビュー（NO-GO）の指摘P2-1（同一IRを返す条件にrootの変換成功が含まれていない）だけを直した。P2-3は、maintainerの`support-sync`を待つ既知の項目として残す。この段では文の置換だけを行い、build、テスト、ベンチマークは走らせていない。

### 指摘と変更

- 具体例。変数宣言のないrootに`Init == TRUE`と`Next == TRUE`を定義して設定で選び、明示v2で有効な未使用moduleを添えると、bundleの検証は通る。実装はそのあと元のrootを変換し、`Unsupported TLA input: state variables`を返す。追加moduleの有無にかかわらず同じエラーになるので、bundleの検証が通っただけではIRは返らない。英語のREADMEでは、直前の連結や設定上書きを要する要求も`such a request`の指示先に読めた。
- 条件を3箇所とも「連結と設定上書きを要さず、bundleの検証が通れば、単体のv2と同じrootの変換を行う。変換に成功した場合は同じIRを返す」に揃えた。`docs/safety-ir-v2.md:5`の「通常どおり変換する」と同じ意味になる。
  - `crates/idealis-core/README.md:19`: 一致する1文だけを置き換えた。`when the input needs no linking or configuration override and bundle validation passes, v2 lowers the same root as a single-module v2 request, and when that lowering succeeds the program is the same as without them`。
  - `crates/rtla-compiler/README.md:45`: 同一IRの1文だけを置き換えた。
  - `crates/rtla-compiler/SUPPORT.md:15`: 同じ約束の1行だけを置き換えた。
- 置換前の文は、3箇所とも置換の前にそれぞれ1回だけ現れることを確かめた。上記以外の文は変えていない。

### 同じ約束を述べる他の文（変えていない）

- `docs/safety-ir-v2.md:5`と`docs/rust-safety.md:85`は「これらを要さない要求は、追加moduleを伴ってもv2で通常どおり変換する」という言い方で、変換の成功を約束していないので、そのままにした。
- `crates/idealis-core/SUPPORT.md:7`と`crates/rtla-compiler/SUPPORT.md:47`は固定診断の条件だけを述べ、同一IRは述べない。
- 他のREADME、SUPPORT、docsに、追加moduleを添えたときの同じIRや同じprogramを約束する文は見つからなかった。

### 確認

- 4ファイルへの`biome check`と`git diff --check d147550c`は通った。
- 作業木の変更・未追跡のファイルに、作業用の絶対path、利用者名、sessionの作業directory名を探す検索は一致しなかった。
