# 0028: 検査方式ごとの保証をbackend契約へ分離する

0027の比較から、explicit complete、有界の反例なし、帰納性を別の保証として扱うversion 1の契約を作る。定義はsrc/contracts.ts、実装はNode/solver非依存のsrc/backend.tsへ置く。既存のSafetyResult、Rust/Wasm ABI、SMT primitiveのsat/unsat APIは変更しない。

## 契約の境界

backend ID/version、model/property ID、assumption一覧、意味論profile、queryを結果へ結びつける。metadataの一致は保証の取り違えを防ぐもので、IDと実際のモデルの対応やsolverの証明を検証するcertificateではない。対応queryはexplicit、bounded、induction obligation。typed unionとwire decoderを持ち、unknown field/version、誤ったquery/結果の組、異なるscopeを拒否する。

explicit complete、bounded-no-counterexample(k)、inductive、obligation-holds、counterexample、empty、inconclusive、errorを区別する。rtlcのdepth/state/work/temporal打切りから、BMCの完全なk step coverageを推測しない。非空の初期集合が確認されていない成功も、通常の検査成功へ昇格しない。失敗した帰納性のobligationが与えるassignmentと、Initから到達するbehavior traceを分ける。

帰納性の集約は、同一model/property/assumptions/意味論/engine版/invariantのobligationだけで行う。初期成立と保存、invariantが対象propertyと異なる場合は含意も要求する。baseの非空性が必要。missing/unknown/error/反例があれば未確定とし、個々の結果を保持する。

## 実装と対照

pureなrunner、wire decoder、帰納性集約と、既存PortableCoreを呼ぶrtlc adapterを追加する。Apalacheのprocess起動やSMT encodingは契約へ入れない。未知のcapabilityは実行せずunsupportedを返す。入力/結果のmetadataとsnapshotを隔離する。

先に共有contractテストを追加してRedを記録する。Node/Vitestで、異なる保証の混同、空集合、資源切れ、solver unknown、cancel、異なるscope、obligation不足、到達性を持たない反例、不正wireを検査する。実Wasmでrtlcを呼び、complete/bounded/empty/違反を確認する。0027の保存した実応答もこの契約へ写し、旧runは編集しない。

browserとpackaged standalone consumerでもNode/solverなしで使えることを確認する。just check/browser、共有テスト変更に伴うmutation、portable test、package、evaluate/replayを実行する。新規eval runへ条件・入力・結果・digestを残す。今回はgenericなITF importerやApalache adapterを実装済みとはしない。
