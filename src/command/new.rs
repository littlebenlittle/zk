use crate::database::Database;
use crate::Result;

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

#[derive(Debug)]
pub enum Error {
    PathExists,
}

/// create a new zettel
pub fn run<T: Database>(args: impl AsRef<Args<T>>) -> Result<()> {
    let args: &Args<T> = args.as_ref();
    let mut database = T::from_config(args.db_cfg.clone())?;
    database.new_zettel(&args.title)?;
    database.commit()?;
    Ok(())
}
