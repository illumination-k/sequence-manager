use std::path::PathBuf;

use serde::{Serialize, Deserialize};

enum SequenceType {
    Protein,
    Cds,
}

impl ToString for SequenceType {
    fn to_string(&self) -> String {
        match self {
            Self::Protein => "protein",
            Self::Cds => "cds"
        }.to_string()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Source {
    URL,
    OneKp,
    Phytozome,
}

impl ToString for Source {
    fn to_string(&self) -> String {
        match self {
            Self::URL => "url",
            Self::OneKp => "onekp",
            Self::Phytozome => "phytozome"
         }.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum LockObject {
    URL{
        source: Source,
        taxonomy_id: u32,
        name: String,
        url: String,
        gzip: bool,
        version: Option<String>
    },
    OneKp{
        source: Source,
        taxonomy_id: u32,
        name: String,
        onekp_id: String,
    },
    Phytozome{
        source: Source,
        taxonomy_id: u32,
        name: String,
        phytozome_version: String,
    }
}

impl LockObject {
    fn to_url(&self) {}
    fn to_path(&self, mut root: PathBuf, sequence_type: SequenceType) -> PathBuf {
        root.push(sequence_type.to_string());
        match self {
            Self::OneKp { source, taxonomy_id: _, name, onekp_id } => {
                root.push(name);
                root.push(source.to_string());
                root.push(onekp_id);
            },
            Self::Phytozome { source, taxonomy_id: _, name, phytozome_version } => {
                root.push(name);
                root.push(source.to_string());
                root.push(phytozome_version);
            },
            Self::URL { source, taxonomy_id: _, name, url: _, gzip, version } => {
                root.push(name);
                root.push(source.to_string());
                if let Some(v) = version {
                    root.push(v);
                }
            }
        }

        root.clone()
    }
}

fn main() {
    println!("Hello, world!");
}
