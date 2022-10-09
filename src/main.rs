mod database;
mod error;
mod zettel;

use error::Error;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Args {
    #[clap(default_value = ".", long)]
    root_dir: PathBuf,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize a new database
    Init,
    /// Create a new zettel
    New(NewArgs),
    /// Sync changes to zettels with the database
    Sync,
}

#[derive(Debug, clap::Args)]
pub struct NewArgs {
    pub title: String,
}

type Result<T> = std::result::Result<T, Error>;
type DateTime = chrono::DateTime<chrono::Local>;

fn main() -> Result<()> {
    let args = Args::parse();
    let mut db = database::yaml::Database::new(args.root_dir)?;
    let mut zk = match db.get_zk() {
        Ok(zk) => zk,
        Err(e) => {
            println!("error reading database file");
            return Err(e);
        }
    };
    match args.cmd {
        Command::Init => {}
        Command::New(args) => {
            use rand::Rng;
            let id: String = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(18)
                .map(char::from)
                .collect();
            let now = chrono::Local::now();
            let zettel = db.new_zettel(&args.title, &id, now)?;
            zk.add(&zettel)?;
            match db.commit(&zk) {
                Ok(()) => {}
                Err(e) => {
                    zk.rm(zettel)?;
                    return Err(e.into());
                }
            }
        }
        Command::Sync => {
            for entry in std::fs::read_dir(std::env::current_dir()?)? {
                let entry: std::fs::DirEntry = entry?;
                let path = entry.path();
                if path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned()
                    .starts_with("_zettel")
                {
                    continue;
                }
                let zettel = zettel::parse_meta_yaml(&path)?;
                if zettel.is_none() {
                    continue;
                }
                let zettel = zettel.unwrap();
                let id = zettel.get(&"id".into());
                if id.is_none() {
                    println!(
                        "metadata for {} does not contain field id; skipping",
                        path.to_str().unwrap()
                    );
                    continue;
                }
                let id = id.unwrap().as_str();
                if id.is_none() {
                    println!(
                        "metadata for {} contains field id but value is not string; skipping",
                        path.to_str().unwrap()
                    );
                    continue;
                }
                let id = id.unwrap();
                let current_meta = zk.zettels.get_mut(id);
                if current_meta.is_none() {
                    println!(
                        "no metadata with id {} for zettel at {}; skipping",
                        id,
                        path.to_str().unwrap(),
                    );
                }
                let current_meta = current_meta.unwrap();
                current_meta.path = path
                    .strip_prefix(&db.root_dir())
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
            }
        }
    }
    db.commit(zk)?;
    Ok(())
}
