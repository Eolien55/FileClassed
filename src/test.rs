#[test]
fn test_expand() {
    use std::collections::HashMap;

    use crate::run;

    let codes: HashMap<String, String> = [
        ("fr", "French"),
        ("hst", "History"),
        ("cnt", "Century"),
        ("1", "one"),
        ("2", "two"),
        ("shone", "Shell One"),
        ("shtwo", "Shell Two"),
    ]
    .iter()
    .map(|tuple| (String::from(tuple.0), String::from(tuple.1)))
    .collect();

    assert_eq!(run::expand(&"{{fr}".to_string(), &codes), "{French");
    assert_eq!(run::expand(&"{fr".to_string(), &codes), "{fr");
    assert_eq!(
        run::expand(&"{fr} {hst} (18th {cnt})".to_string(), &codes),
        "French History (18th Century)"
    );
    assert_eq!(
        run::expand(&"{fr {hst} (18th {cnt})".to_string(), &codes),
        "{fr History (18th Century)"
    );

    assert_eq!(
        run::expand(&run::expand(&"{sh{1}}".to_string(), &codes), &codes),
        "Shell One"
    );
}
