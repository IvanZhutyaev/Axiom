use aql_compile::compile;
use avm_runtime::{Interpreter, Value};
use std::collections::HashMap;

#[test]
fn compile_and_interpret_filter() {
    let src = r#"source "s"
|> filter(x > 1.0)
|> sink "o""#;
    let c = compile(src).unwrap();
    let mut vm = Interpreter::new(c.module);
    let mut m = HashMap::new();
    m.insert("x".into(), Value::Float(2.0));
    vm.push_event(Value::Struct(m));
    let res = vm.run().unwrap();
    assert!(!res.emitted.is_empty() || res.halted);
}
