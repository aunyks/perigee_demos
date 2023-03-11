pub trait FromConfig {
    type Config<'a>;

    fn from_config<'a>(config: Self::Config<'a>) -> Self;

    fn set_config<'a>(&mut self, _config: Self::Config<'a>) {}
}
