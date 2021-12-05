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
        run::expand(
            &"{{fr}".to_string(),
            &codes,
            '{',
            '}',
            None,
            &mut vec![],
            ','
        ),
        "{French"
    );
    assert_eq!(
        run::expand(&"{fr".to_string(), &codes, '{', '}', None, &mut vec![], ','),
        "{fr"
    );
    assert_eq!(
        run::expand(
            &"{fr} {hst} (18th {cnt})".to_string(),
            &codes,
            '{',
            '}',
            None,
            &mut vec![],
            ','
        ),
        "French History (18th Century)"
    );
    assert_eq!(
        run::expand(
            &"{fr {hst} (18th {cnt})".to_string(),
            &codes,
            '{',
            '}',
            None,
            &mut vec![],
            ','
        ),
        "{fr History (18th Century)"
    );

    assert_eq!(
        run::expand(
            &run::expand(
                &"[sh{1}]".to_string(),
                &codes,
                '{',
                '}',
                None,
                &mut vec![],
                ','
            ),
            &codes,
            '[',
            ']',
            None,
            &mut vec![],
            ','
        ),
        "Shell One"
    );

    assert_eq!(
        run::expand(
            &run::expand(
                &"{sh{2}}".to_string(),
                &codes,
                '{',
                '}',
                Some(3),
                &mut vec![],
                ','
            ),
            &codes,
            '{',
            '}',
            None,
            &mut vec![],
            ','
        ),
        "Shell Two"
    );

    assert_eq!(
        run::expand(
            &run::expand(
                &"{,{2}}",
                &codes,
                '{',
                '}',
                None,
                &mut vec!["sh".to_string()],
                ','
            ),
            &codes,
            '{',
            '}',
            None,
            &mut vec!["sh".to_string()],
            ','
        ),
        "Shell Two"
    );
}

#[test]
fn test_decode() {
    use crate::run;

    let history: Vec<_> = vec!["fr", "hst", "cnt", "1", "2", "shone", "shtwo"]
        .iter()
        .map(|x| x.to_string())
        .collect();

    assert_eq!(run::expand_last(&",,".to_string(), &history, ','), "hst");

    assert_eq!(run::expand_last(&"fr".to_string(), &history, ','), "fr");

    assert_eq!(
        run::expand_last(&",,,,,,,,".to_string(), &history, ','),
        "shtwofr"
    );
    assert_eq!(run::expand_last(&",aa".to_string(), &history, ','), "fraa");
    assert_eq!(run::expand_last(&"".to_string(), &history, ','), "");
}
