use serde_json::Value;
use tezbar_convert::engine;
#[test]
fn executed_tsx_corpus() {
    let corpus: Vec<Value> = serde_json::from_str(include_str!("../qa/oracle.json")).unwrap();
    let mut errors = vec![];
    for (i, c) in corpus.iter().enumerate() {
        let rows = engine::convert(
            c["query"].as_str().unwrap(),
            c["preferences"].get("basePixel"),
        );
        let got = serde_json::to_value(rows).unwrap();
        if got != c["rows"] {
            errors.push(format!(
                "case {i} {} pref {}\n got {got}\nwant {}",
                c["query"], c["preferences"], c["rows"]
            ));
        }
    }
    assert!(
        errors.is_empty(),
        "{} differences:\n{}",
        errors.len(),
        errors
            .iter()
            .take(15)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
