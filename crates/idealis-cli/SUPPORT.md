# idealis-cli の対応範囲

- [x] 標準入力の各行を同一の idealis-connect Engine へ渡し、各応答を JSON と改行で標準出力へ書き、毎回 flush する。
- [x] モデルの許容値と各実装の観測を渡し、応答から対象ごとの照合結果を取得できる。
- [ ] SUT プロセスの起動、モデルファイルの読み込み、観測の自動収集。

[実行口](src/main.rs)と[差分の入出力テスト](tests/jsonl.rs)を参照。
