use anyhow::{anyhow, Result};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use flate2::bufread::GzDecoder;
use serde::{Deserialize, Serialize};
use tar::Archive;

pub mod sql;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyRelation {
    id: u32,
    rank: TaxonomyRank,
}

/// NCBI Taxonomy
/// Using only scientific name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyRecord {
    id: u32,
    parent_id: u32,
    rank: TaxonomyRank,
    scientific_name: String,
    children: Vec<TaxonomyRelation>,
}

impl TaxonomyRecord {
    fn init(id: u32, parent_id: u32, rank: TaxonomyRank) -> Self {
        Self {
            id,
            parent_id,
            rank,
            scientific_name: String::new(),
            children: Vec::new(),
        }
    }

    fn new(
        id: u32,
        parent_id: u32,
        rank: TaxonomyRank,
        scientific_name: String,
        children: Vec<TaxonomyRelation>,
    ) -> Self {
        Self {
            id,
            parent_id,
            rank,
            scientific_name,
            children,
        }
    }
}

/// Only use basic taxonomy Level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum TaxonomyRank {
    NoRank,
    Kingdom,
    Class,
    Order,
    Clade,
    Family,
    Genus,
    Species,
    SubSpecies,
    UnusedRanks(String),
}

impl ToString for TaxonomyRank {
    fn to_string(&self) -> String {
        match self {
            TaxonomyRank::NoRank => "norank",
            TaxonomyRank::Kingdom => "kingdom",
            TaxonomyRank::Class => "class",
            TaxonomyRank::Order => "order",
            TaxonomyRank::Clade => "clade",
            TaxonomyRank::Family => "family",
            TaxonomyRank::Genus => "genus",
            TaxonomyRank::Species => "species",
            TaxonomyRank::SubSpecies => "subspecies",
            TaxonomyRank::UnusedRanks(s) => s,
        }
        .to_string()
    }
}

impl From<&str> for TaxonomyRank {
    fn from(value: &str) -> Self {
        match value {
            "norank" => TaxonomyRank::NoRank,
            "kingdom" => TaxonomyRank::Kingdom,
            "class" => TaxonomyRank::Class,
            "order" => TaxonomyRank::Order,
            "clade" => TaxonomyRank::Clade,
            "family" => TaxonomyRank::Family,
            "genus" => TaxonomyRank::Genus,
            "species" => TaxonomyRank::Species,
            "subspecies" => TaxonomyRank::SubSpecies,
            _ => TaxonomyRank::UnusedRanks(value.to_string()),
        }
    }
}

impl From<String> for TaxonomyRank {
    fn from(value: String) -> Self {
        TaxonomyRank::from(value.as_str())
    }
}

fn read_nodes(path: &Path, recs: &mut HashMap<u32, TaxonomyRecord>) -> Result<()> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let attrs: Vec<String> = line?.split('|').map(|a| a.trim().to_string()).collect();
        let id = attrs[0].parse::<u32>()?;
        let parent_id = attrs[1].parse::<u32>()?;
        let rank = TaxonomyRank::from(attrs[2].to_owned());
        recs.entry(id)
            .or_insert(TaxonomyRecord::init(id, parent_id, rank));
    }

    let _recs: Vec<TaxonomyRecord> = recs.values().map(|k| k.to_owned()).collect();
    let mut graph: HashMap<u32, Vec<u32>> = HashMap::new();

    for (child_id, parent_id) in recs.values().map(|k| (k.id, k.parent_id)) {
        graph
            .entry(parent_id)
            .or_insert_with(Vec::new)
            .push(child_id);
    }

    fn get_all_child(
        start: &u32,
        children: &mut Vec<u32>,
        graph: &HashMap<u32, Vec<u32>>,
        memo: &mut HashMap<u32, Vec<u32>>,
        recs: &HashMap<u32, TaxonomyRecord>,
    ) {
        let cur_children = if let Some(c) = graph.get(start) {
            c
        } else {
            return;
        };
        children.extend(cur_children.iter().cloned());

        for c in cur_children.into_iter() {
            if let Some(memo_children) = memo.get(c) {
                children.extend(memo_children.iter().cloned());
                continue;
            };
            get_all_child(c, children, graph, memo, recs);
        }
    }

    let mut memo = HashMap::new();
    for (i, rec) in _recs.iter().enumerate() {
        if i % 100000 == 0 && i != 0 {
            println!("now process: {}", i)
        }

        if rec.rank > TaxonomyRank::Clade {
            continue;
        }

        let mut children = vec![];
        get_all_child(&rec.id, &mut children, &graph, &mut memo, &recs);
        memo.entry(rec.id).or_insert(children.clone());

        let children = children
            .iter()
            .map(|id| TaxonomyRelation {
                id: *id,
                rank: recs.get(id).unwrap().rank.clone(),
            })
            .collect();

        if let Some(r) = recs.get_mut(&rec.id) {
            r.children = children;
        }
    }

    println!("finish reading nodes.dmp");

    Ok(())
}

fn read_names(path: &Path, recs: &mut HashMap<u32, TaxonomyRecord>) -> Result<()> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let attrs: Vec<String> = line?.split('|').map(|a| a.trim().to_string()).collect();
        let id = attrs[0].parse::<u32>()?;

        if attrs[3] != "scientific name" {
            continue;
        }

        let name = attrs[1].to_owned();

        if let Some(rec) = recs.get_mut(&id) {
            rec.scientific_name = name;
        } else {
            return Err(anyhow!("Get unknown taxid {} from {}", id, path.display()));
        };
    }
    Ok(())
}

fn _load_taxonomy_from_dump(nodes: &Path, names: &Path) -> Result<HashMap<u32, TaxonomyRecord>> {
    let mut recs: HashMap<u32, TaxonomyRecord> = HashMap::new();

    read_nodes(nodes, &mut recs)?;
    read_names(names, &mut recs)?;

    Ok(recs)
}

fn downlaod_taxonomy_dump(path: &Path) -> Result<()> {
    let resp = reqwest::blocking::get("https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz")?;
    let br = BufReader::new(resp);
    let tarfile = GzDecoder::new(br);
    let mut archive = Archive::new(tarfile);
    archive.unpack(path)?;
    Ok(())
}

#[cfg(test)]
mod test_taxonomy {
    use std::path::PathBuf;

    use crate::{_load_taxonomy_from_dump, downlaod_taxonomy_dump};
    use anyhow::Result;

    #[test]
    fn test_download_and_load() -> Result<()> {
        downlaod_taxonomy_dump(&PathBuf::from("./_tmp"))?;
        let recs = _load_taxonomy_from_dump(
            &PathBuf::from("./_tmp/nodes.dmp"),
            &PathBuf::from("./_tmp/names.dmp"),
        )?;
        Ok(())
    }
}
