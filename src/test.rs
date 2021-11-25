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

    assert_eq!(
        run::expand(&"{{fr}".to_string(), &codes, '{', '}', None),
        "{French"
    );
    assert_eq!(
        run::expand(&"{fr".to_string(), &codes, '{', '}', None),
        "{fr"
    );
    assert_eq!(
        run::expand(
            &"{fr} {hst} (18th {cnt})".to_string(),
            &codes,
            '{',
            '}',
            None
        ),
        "French History (18th Century)"
    );
    assert_eq!(
        run::expand(
            &"{fr {hst} (18th {cnt})".to_string(),
            &codes,
            '{',
            '}',
            None
        ),
        "{fr History (18th Century)"
    );

    assert_eq!(
        run::expand(
            &run::expand(&"[sh{1}]".to_string(), &codes, '{', '}', None),
            &codes,
            '[',
            ']',
            None
        ),
        "Shell One"
    );

    assert_eq!(
        run::expand(
            &run::expand(&"{sh{2}}".to_string(), &codes, '{', '}', Some(3)),
            &codes,
            '{',
            '}',
            None
        ),
        "Shell Two"
    );
}
