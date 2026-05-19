use aql_syntax::parse;
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_simple_filter_does_not_panic(t in 0.0f64..100.0) {
        let src = format!(
            r#"source "s"
|> filter(x > {t})
|> sink "o""#
        );
        let _ = parse(&src);
    }

    #[test]
    fn parse_source_sink_roundtrip(name in "[a-z][a-z0-9_]{0,12}") {
        let src = format!(r#"source "{name}" |> sink "{name}""#);
        let prog = parse(&src).expect("valid pipeline");
        prop_assert_eq!(prog.stages.len(), 2);
    }
}
