use runestick::{Context, FromValue, Module, Source};
use std::sync::Arc;

fn divide_by_three(value: i64) -> i64 {
    value / 3
}

#[tokio::main]
async fn main() -> runestick::Result<()> {
    let mut my_module = Module::new(&["mymodule"]);
    my_module.inst_fn("divide_by_three", divide_by_three)?;

    let mut context = Context::with_default_modules()?;
    context.install(&my_module)?;

    let options = rune::Options::default();
    let mut warnings = rune::Warnings::disabled();

    let unit = rune::load_source(
        &context,
        &options,
        Source::new(
            "test",
            r#"
            fn main(number) {
                number.divide_by_three()
            }
            "#,
        ),
        &mut warnings,
    )?;

    let vm = runestick::Vm::new(Arc::new(context), Arc::new(unit));
    let output = vm.call(&["main"], (33i64,))?.complete()?;
    let output = i64::from_value(output)?;

    println!("output: {}", output);
    Ok(())
}
