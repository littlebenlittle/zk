use crate::Result;
use crate::database::Database;

#[derive(Debug, clap::Args)]
pub struct Args<T: Database> {
    /// database config
    db_cfg: T::Config,
    /// title for new zettel
    title: String,
}

impl<T: Database> AsRef<Self> for Args<T> {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

/// sync changes to local files with database
pub fn run<T: Database>(args: impl AsRef<Args<T>>) -> Result<()> {
    unimplemented!()
}
