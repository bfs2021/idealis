# 0062: 共有IRの統合と最新版の比較

umbrella [#63](https://github.com/mizchi/idealis/issues/63) のP0を実行する。比較・profileは [#64](https://github.com/mizchi/idealis/issues/64)、共有IRは [#60](https://github.com/mizchi/idealis/issues/60) で追跡する。基準はローカル `3f249cc7`。共有IRの実装はdraft PR #62の `a89e1d9cbf0260f2be24d898cab60daaf20e3f8e` をローカルへ統合し、重複実装を避ける。remoteのmerge・pushは行わない。

## 仮説と契約

共有IR v2はoperatorのinline膨張とcaller/calleeのbound slot混線を解消する。主指標は受理範囲、IR bytes、compile時間、RSS。実行時の指数仕事の除去とは数えない。v1を既定のまま保ち、既存の正常・故障・拒否・budget結果とSYMMETRY/代表展開/PORを回帰検査する。v2のmodule連結非対応、action外殻の非共有、独立した深さ上限は明記する。

call-by-name、caller lexical frame、未使用引数、prime/ENABLED、capture、work課金はTLCの実装・保存oracleと照合する。v2のcall/frame課金によるwork差を隠さない。縮約未対応のv2は安全な拒否または既存条件に従う通常探索とし、誤って縮約しない。深いIRのdrop変更もv1の速度・破棄費用と小さいstackで検査する。

## 順序とRed

1. このplanを先にcommitする。0061-finalのsource/binary/digestをbaselineとして保持する。
2. #62のテスト・実装をレビューし、現行での共有IR拒否をRedとして保存する。既存のv1応答、SYMMETRY/PORを対照にする。
3. 固定headをローカル統合する。衝突を解消し、v2の深いchain、binder/capture、lazy引数、prime/ENABLED、不正IR、work境界、TS/Rust構造、native/Wasm応答を検査する。保存したQuint生成物は改変しない。
4. #62のanalysis parityが通常のtask/CIから漏れている点を修正し、実行可能な前提buildをtaskに記す。既存のv1 fixture・回帰を保持する。
5. 正しさを確認した候補をfreezeし、build/testと別に通常時間・RSS・profileを採取する。全失敗を新しいrunへ保存する。

## 比較条件

同じTLA/config、maxDepth=10000000、maxStates=10000000、maxWork=4000000000で完全探索を比較する。既存function N20とchannel N7はtla-ts 1、Java TLC 1/8、baseline/candidate native 1/8、baseline/candidate Node Wasm 1の9経路を使う。各1 warmup + 3 samples、経路順を回転・反転する。小さい4モデルの正常/故障も全経路で検査する。状態数・生成数・verdictが一致しない時間を速度比較に使わない。

公開実モデルの対照には既存Quintの6生成物と正常/故障設定を使う。record・集合・循環は既存の縮約なし小モデルとSYMMETRYの対照を使う。小さい入力のcold時間を探索kernelの速度とは呼ばない。IR v2のcompile bytes/定義数/call数、compile/checkの費用はv1と別に報告し、v1拒否を無理にinlineしない。

prepared kernelは既存14本のCriterionを同じbinary設定でbefore/after/after/beforeで比較する。20 samples、warmup 0.5秒、measurement 1秒。共有IRと削減の追加対照は既存cohortを再実行する。profileはnamed release binary、samplyの既存driverで通常時間から分ける。compile/check、式評価、値intern/update、visited、merge/accept/invariant、dropの観測可能な内訳を記録し、sample比をwall比と混同しない。

OS・CPU・Node/Rust/Java・jar・tla-ts version、source/binary/inputのhash、workers、順序、全sample、RSS、raw profileを保存する。hostは専用機ではない。変動・10%以上の後退・未対応・実行失敗を残し、結果後に条件を変えるときは追記してから新規runにする。既存0061の速度を新しいrunへ混ぜない。

## 採否と完了条件

共有IRの採用はTLCの意味一致、v1の互換性、inline膨張・slot混線の解消を前提にする。v1の性能後退があれば原因と大きさを明記し、有用性と分けて判断する。最新比較とprofileから#63のP1の順を更新する。測っていない私有モデルや全TLA+への保証を主張しない。

終了前にcheck/browser、portable-build/test、compiler-test、safety-test、quint-test、analysis parity、runtime-build/test、fmt/clippy、evaluate/replay-evaluation、packageを実行する。共有TSテスト変更時はmutation。SUPPORT/Jevを同期しoffline/staged検査を通す。キーなしのskipは検査済みと区別する。結果と制限をiteration、README、#64と#63へ反映する。#60/#62のremote mergeやcloseは行わない。
