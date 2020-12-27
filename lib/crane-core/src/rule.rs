use super::Label;

/// A Rule defines what actions to take to perform some work.
///
/// Some examples of rules are `ErlangLibrary` or `ElixirTest`.
///
pub trait Rule {
    fn name(&self) -> &str;

    fn toolchains(&self) -> Vec<Label>;

    /// The execution of this rule is captured in this function.
    ///
    /// For `*_library` rules this will be build steps, for `*_test` rules this will be execution.
    ///
    fn execute(&mut self) -> Result<(), anyhow::Error>;
}
