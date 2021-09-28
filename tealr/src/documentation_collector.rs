///This trait is used to collect the documentation of exposed functions/methods.
///It can be used to enhance the generated .d.tl file, as well as generate a `:help()` method.
pub trait DocumentationCollector {
    ///add documentation to the function/method
    fn document_function(&mut self, name: impl Into<String>, documentation: impl Into<String>);
    ///should this type get a `:help()` method
    fn should_generate_help_method(&mut self, should_generate: bool);
}
pub trait HelpMethodGenerator {
    fn generate_help(&mut self);
}
