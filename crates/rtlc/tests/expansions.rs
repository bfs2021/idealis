use rtlc::{SafetyBounds, SafetyExpansionCache, SafetyProgram};
use serde_json::{Value, json};

fn program() -> SafetyProgram {
    serde_json::from_value(json!({
        "format":"modelt.safety-program", "version":1, "name":"CacheCycle", "variables":["x"],
        "initial":[{"name":"Init","steps":[{"kind":"assign","slot":0,"value":{"op":"literal","value":{"type":"int","value":"0"}}}]}],
        "next":[{"name":"Next","steps":[{"kind":"assign","slot":0,"value":{"op":"if","condition":{"op":"lt","left":{"op":"read","slot":0,"next":false},"right":{"op":"literal","value":{"type":"int","value":"3"}}},"consequent":{"op":"add","left":{"op":"read","slot":0,"next":false},"right":{"op":"literal","value":{"type":"int","value":"1"}}},"alternative":{"op":"literal","value":{"type":"int","value":"0"}}}}]}],
        "invariants":[{"name":"Safe","expression":{"op":"lt","left":{"op":"read","slot":0,"next":false},"right":{"op":"literal","value":{"type":"int","value":"4"}}}}],
        "assumptions":[],"checkDeadlock":true
    })).unwrap()
}
fn bounds() -> SafetyBounds {
    SafetyBounds {
        max_depth: 16,
        max_states: 100,
        max_work: 100000,
        max_temporal_nodes: None,
    }
}

#[test]
fn persisted_expansions_recheck_a_changed_invariant() {
    let program = program();
    let mut cache = SafetyExpansionCache::default();
    let cold = program
        .check_incremental(bounds(), &mut cache, &"a".repeat(64), 100)
        .unwrap();
    assert_eq!(cold.result.status, "complete");
    assert_eq!(cold.stats.hits, 0);
    assert_eq!(cold.stats.writes, 4);
    let mut cache = serde_json::from_slice(&serde_json::to_vec(&cache).unwrap()).unwrap();
    let warm = program
        .check_incremental(bounds(), &mut cache, &"a".repeat(64), 100)
        .unwrap();
    assert_eq!(warm.stats.hits, 4);
    assert_eq!(warm.stats.misses, 0);
    assert_eq!(warm.result.generated, cold.result.generated);
    let mut changed = serde_json::to_value(&program).unwrap();
    changed["invariants"][0]["expression"]["right"]["value"]["value"] = json!("2");
    let changed: SafetyProgram = serde_json::from_value(changed).unwrap();
    let checked = changed
        .check_incremental(bounds(), &mut cache, &"a".repeat(64), 100)
        .unwrap();
    assert_eq!(checked.result.status, "invariant-violation");
    assert_eq!(checked.stats.hits, 2);
    let mut expected = changed.check(bounds()).unwrap();
    expected.work = checked.result.work;
    assert_eq!(checked.result, expected);
}

#[test]
fn interrupted_expansions_are_not_saved_as_complete() {
    let program = program();
    let mut cache = SafetyExpansionCache::default();
    let result = program
        .check_incremental(
            SafetyBounds {
                max_states: 1,
                ..bounds()
            },
            &mut cache,
            &"a".repeat(64),
            100,
        )
        .unwrap();
    assert_eq!(result.result.status, "bounded");
    assert_eq!(result.stats.writes, 0);
    let result = program
        .check_incremental(bounds(), &mut cache, &"a".repeat(64), 100)
        .unwrap();
    assert_eq!(result.stats.hits, 0);
    assert_eq!(result.result.status, "complete");
    let json: Value = serde_json::to_value(&cache).unwrap();
    assert_eq!(json["entries"].as_object().unwrap().len(), 4);
}
