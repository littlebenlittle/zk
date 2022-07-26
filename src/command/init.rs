use crate::{
    database::{Database},
    Result,
};

#[derive(Debug, clap::Args)]
pub struct Args<T: Database> {
    /// database config
    db_cfg: T::Config,
}

impl<T: Database> AsRef<Self> for Args<T> {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

/// Initialize a new zk database
pub fn run<T: Database>(args: impl AsRef<Args<T>>) -> Result<()> {
    let args: &Args<T> = args.as_ref();
    let mut database = T::from_config(args.db_cfg.clone())?;
    database.init()?;
    database.commit()?;
    Ok(())
}
