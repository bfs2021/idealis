# 0041: SMT-LIB scriptの問い合わせ列

0040のcontext APIを使い、SMT-LIB入力のpush/pop・複数check-sat・Bool literalのcheck-sat-assumingを追加する。冒頭にInt/Boolの宣言と既存のset-logic/set-optionを置く範囲に限定する。assert・push/pop・checkの後の宣言やoption変更は拒否する。scope内の宣言、define-fun、get-model/get-value、reset、exit、一般LIA、学習状態の再利用は今回含めない。

公開Rust APIはScript::parse(source)とScript::solve(max_work)を追加する。parseで全コマンド・型・scope・assumptionsを検査し、各check時点の式を保持する。失敗したスクリプトから部分的なSATを返さない。checkのない入力は空の結果列で、安全性や完全性の証明として数えない。既存Problem/SolverとJSON solve/prepare/checkの契約は保つ。

JSONはrun-script {source,maxWork}を追加し、script-results {results:[Solution,...]}を返す。結果順はcheckの順。maxWorkは全checkで共有し、各queryのstats.workを差し引く。残り0のcheckはunknown/workで、モデルを返さない。構文・型・資源エラーはwork 0でもerrorにする。各checkは探索をやり直し、再開や学習の再利用を主張しない。

checkは最大256件、scopeとassumptionsは各256件。既存のsource/token/深さ上限を使い、push/popはcanonicalなu32 numeralだけを受け付ける。0はno-op。assumptionsは宣言済みBool symbolか(not symbol)だけとし、任意Bool式は対象外と明記する。全体の解析が失敗しても、既存のEngine handleに影響させない。

SMT-LIB 2.7参照文書のassertion stackとcheck-sat-assumingの定義、およびZ3 5.4.1の実行で確認する。Z3の逐次script判定と、TS側で独立に保持した各checkのassertion stackから作るsnapshotの判定を比較する。全SATモデルをsnapshotに固定してZ3へ再照合する。native/Wasmの順序・判定・モデル・work、unknown/error/空結果の区別を検査する。

回帰seed 0x4101、開発seed 0x41a1の32 script、確認seed 0x41c3の64 scriptを予約する。各生成scriptでSATとUNSAT、push/pop、正負の一時assumptionsと復元を含める。手書き境界と不正入力も検査する。確認は同じsourceとbinaryを使い、入力・Z3出力・snapshot・全応答・source/lock/binary hashを新規runへ保存し再生する。0040のnative/Wasmをbaselineとして保存し、旧版の未対応を確認する。人工入力の対応率を実用入力全体や速度へ一般化しない。

TDDのRed/Green、Rust fmt/clippy/tests、node:test/Vitest、実Chromium、既存0040とSMT差分、check/browser、runtime-build/test、evaluate/replay、packageの独立consumerを確認する。古いrunを変更せず、条件を変えた場合は追記のplanと新runを作る。PR #49へまとめ、mergeとregistry publishは行わない。
