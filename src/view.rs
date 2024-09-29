use std::io::Stdout;
use crate::Result;

pub(crate) trait View {
    fn display(&self, stdout: &mut Stdout) -> Result<()>;
}
