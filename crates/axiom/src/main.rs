//! Main entry point — delegates to `axiom-node`.

fn main() -> anyhow::Result<()> {
    axiom_node::run()
}
