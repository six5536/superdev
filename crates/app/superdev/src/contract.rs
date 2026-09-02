//! contract.rs — the CLI contract's framework rule, bound to the command tree.
//!
//! The contract's definition is the `cli` regions the binary's sources carry,
//! materialised into `contract-002-cli-superdev` by `validate --fix` and
//! failed by `validate` when stale (ADR-042), so nothing here compares a copy.
//! What remains is the one promise the regions cannot show: the framework
//! adds `--help` to every command and `--version` to the root, and the
//! contract states that rule once instead of repeating it per entry.

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    /// Covers I035 criterion 6: every command carries the framework's help
    /// flag and the root its version flag, which is the rule the contract
    /// states once instead of repeating in every entry. The framework adds
    /// both while building, so what binds is that neither is disabled.
    #[test]
    fn every_command_carries_the_help_flag() {
        fn check(command: &clap::Command, path: &str) {
            assert!(
                !command.is_disable_help_flag_set(),
                "`{path}` disables --help, and the contract states every command carries it"
            );
            for sub in command.get_subcommands() {
                check(sub, &format!("{path} {}", sub.get_name()));
            }
        }
        let root = crate::Cli::command();
        check(&root, "superdev");
        assert!(
            !root.is_disable_version_flag_set(),
            "the root disables --version, and the contract states it carries it"
        );
    }
}
