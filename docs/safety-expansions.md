# 状態ごとのNext評価を再利用する

`rtlc`の`check_incremental`は、前回評価を完了したNextのsuccessor列を使い、同じ状態のNext評価を省く。初期状態、visited set、propertyの合否は保存しない。現在のInitから毎回探索し、invariant、action property、constraints、deadlockとtemporal/fairnessを既存の検査経路で評価する。

通常の`check`と`check_parallel`はcacheを使わない。新しい経路はnative/Wasmとも逐次探索。状態展開を複数workerへ分配するBFS coordinatorや、進行中queueのcheckpoint再開は含めない。

cacheは既定で無効。TS/JSONLは`enabled: true`、Rustは`ExpansionOptions.enabled = true`で有効にする。省略またはfalseなら通常の`check`へ委譲し、cacheを参照・更新せず、hit/miss/writeを0で返す。保存済みcacheがあっても自動的に有効にはならない。

```ts
import {
  createSafetyExpansionCache,
  checkSafetyIncremental,
} from "@mizchi/idealis/portable";

const cold = checkSafetyIncremental(core, program, bounds, {
  enabled: true,
  cache: createSafetyExpansionCache(),
  buildDigest,
  maxEntries: 10_000,
});
const next = checkSafetyIncremental(core, changedPropertyProgram, bounds, {
  enabled: true,
  cache: cold.cache,
  buildDigest,
  maxEntries: 10_000,
});
console.log(next.result.status, next.stats);
```

`buildDigest`は実際のbackendと、意味に影響する依存を固定するSHA-256。Wasm例では実bytesのdigestを渡す。nativeとWasmは異なるbinaryなので、byteのdigestを使う場合は相互にhitさせない。wire形式は同じで、同じnative binaryの別processや同じWasm bytesの別instanceへsnapshotを渡せる。

Rustでは`SafetyExpansionCache::default()`を作り、`program.check_incremental(bounds, &mut cache, ExpansionOptions { enabled: true, build_digest, max_entries })`を呼ぶ。`ExpansionOptions::default()`は無効。`result`と今回の`stats`を返し、有効時はcacheを更新する。`serde_json`でsnapshotを保存・復元できる。JSONL/Wasmのrequestは`check-safety-incremental`、responseは`incremental-safety`。

## 保存・失効する条件

namespaceにはIR version、変数の順序、Next全体、全definitionとbuild digestを含める。Init、property、constraints、探索上限は除く。これらは毎回評価する。Nextの評価に影響するconstants/moduleの違いは、コンパイルされたNext/definitionに反映される。v2でpropertyの変更がdefinition集合も変える場合は保守的に全missになる。

状態のkeyは正規化したportable値のJSON全文。`Scalar::Ref`、process内のintern IDやfingerprintは保存しない。各entryはaction indexとsuccessorの順序・重複を保つ。constraintsで除外する前の候補を保存するので、制約の緩和によって新たに到達する状態も探索できる。

全branchの評価を完了したentryだけを書き込む。work/state予算切れ、反例や評価エラーで中断した展開は保存しない。depth予算でtargetをenqueueしなくても、Next列挙を最後まで完了したentryは再利用できる。run全体が違反・boundedでも、別の状態で先に完了した展開を破棄する必要はない。

successorが空のentryも完全な評価結果として保存する。現在の`checkDeadlock`を毎回適用するため、以前deadlock検査を無効にしていても新しいrunでは違反を検出する。

`maxEntries`は保持する状態数の上限。0では保存しない。満杯では新しいentryを書かず、小さい上限へ変更するとkey順の末尾から落とす。successor数・byte数・RSSの厳密な上限ではない。今回の`work`には実行した処理だけを計上し、過去のNext評価のworkは足さない。hitでも値の復元とproperty検査の費用は残る。

## shardのsnapshotをまとめる

同じnamespaceのcacheは`mergeSafetyExpansions(core, left, right, maxEntries)`、Rustでは`cache.merge(other, max_entries)`で統合できる。同じ状態について異なるsuccessor列があれば拒否する。entryのdigest、namespaceとidentity全文を検査し、壊れたsnapshotを成功へ読み替えない。snapshotのmergeは、分配した全taskの完了やモデルの安全性を意味しない。

snapshotはホストが輸送・永続化する。共有storeへ公開する場合のCAS・所有権は[task cacheの契約](check-cache.md)と同じくホストが用意する。このAPI自体は分散lockを取らない。

SHA-256は偶発的な改変の検出用で、生成者の署名ではない。正しい生成器が完全なsuccessor列を書いたことと、意味に影響する依存をnamespaceに含めたことは前提。再署名された偽のentryを意味論的に証明する機能は持たない。

## 証明と検証

[`Cache.lean`](../formal/cache/Cache.lean)へNext再利用の9定理と、無効時に再利用しない1定理を加えた。有効化され、build・Next・状態が同じでcompleteなentryだけを使う条件と、保存関係が現在のNextと一致するなら任意の現在Initからの到達関係が保たれること、現在のpropertyを再検査すれば安全性が従うことを示す。propertyは以前のrunと同じである必要がない。

証明は関係とpolicyのモデルを対象にする。Rustの全実装やSHA-256、TLA+ evaluatorの機械証明ではない。有効化の有無を含む32通りのcache利用条件を実Rust/Wasmで作り、Leanの実行可能な参照機へ照合する。TLCオラクルを持つQuint生成物、constraint境界、予算切れ、複合値、action property、temporal/fairness、破損とmerge競合も実装で検査する。

```sh
just expansion-test
just expansion-eval .artifacts/new-expansion-eval
just expansion-replay .artifacts/new-expansion-eval .artifacts/new-expansion-replay
just expansion-proof .artifacts/new-expansion-proof .artifacts/new-expansion-eval/reuse-probes.json
just expansion-bench .artifacts/new-expansion-bench unique-baseline-name
just expansion-bench-replay .artifacts/new-expansion-bench
just browser
```

Criterionは通常探索、初回保存、native objectの再利用、JSON snapshotの復元・保存を分ける。compileとprocess起動は含めない。小さいモデルの後退も含めた採用判断は[評価記録](../evals/iterations/cached-expansions.md)へ残す。
