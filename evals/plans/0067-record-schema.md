# 0067: 同じrecord値のschema再検査

[#69](https://github.com/mizchi/idealis/issues/69)。基準は4587a3ea。SYMのnode行を作る際、先頭recordのschemaを検査してから各nodeのschemaを組み立て直している。完全に同じScalarのrecordだけは再構築を省く。shape.lenの論理workは従来と同じ順で課金し、異なるrecordのschema・型検査とエラーを維持する。Arenaをまたぐcache・新しい共有metadata・縮約条件の拡張は加えない。

旧版で同一recordのnode関数について、key・先頭代表とkey/代表の論理workを先に検査する。TLCStateMutの具体値と代表の分離、FcnRcdValueのimmutableな値の扱いを照合する。既存の異なるschema・node参照・typed値の拒否も保つ。

同じsymmetry benchとfixtureを使い、before/afterのbinary・全sourceを固定する。最初はrecord N12の全展開/代表展開と1/8 workerの4本をABBA、20 samples、warmup 0.5秒、measurement 1秒で測る。build/test/profileを重ねない。約5%以上のnative短縮を採用根拠とし、小さい差なら固定binaryの4本を一度だけ再確認する。追加で5%へ届かなければproductionの変更を撤去する。10%以上の後退も固定binaryで確認する。

有用なら21本のSYM/通常14本とsource境界・TLCの対照、必要な全体検査へ広げる。撤去する場合はsafety_symmetry.rsのproduction部分が4587a3eaと同じことを確認し、追加した挙動testとcheck/browser、fmt/clippy、portable、runtime build/test、evaluate/replay、SUPPORT/Jevを確認する。rawは追記のみ。不採用のsource・全sample・論理workの対照を保存する。

これを今回の最後の小さい仮説とする。shape cache・borrowed entries・小整数圧縮・worker制御の拡張は、実モデルで支配費用やRSS制約を示せるまで保留する。残る費用を推測だけで高速化と呼ばず、採否と再開条件をiterationと#69/#63へ記録する。
