use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Acquire, QueryBuilder, Sqlite, SqlitePool};

use crate::TaxonomyRecord;

pub async fn setup_db() -> Result<SqlitePool> {
    let db_uri = std::env::var("DATABASE_URL")?;

    if !Sqlite::database_exists(&db_uri).await? {
        Sqlite::create_database(&db_uri).await?;
    }
    let pool = SqlitePool::connect(&db_uri).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}

const SQLITE_LIMIT: usize = 32766;

pub async fn insert_taxonomies(pool: &SqlitePool, taxonomies: &[TaxonomyRecord]) -> Result<()> {
    let mut conn = pool.acquire().await?;
    let mut tx = conn.begin().await?;

    // taxonomy record insertion
    {
        for chunk in taxonomies.chunks(SQLITE_LIMIT / 3) {
            let mut query_builder: QueryBuilder<Sqlite> =
                QueryBuilder::new("INSERT INTO taxonomy_record(id, name, rank) ");
            query_builder.push_values(chunk, |mut b, taxonomy| {
                b.push_bind(taxonomy.id.to_string())
                    .push_bind(taxonomy.scientific_name.to_string())
                    .push_bind(taxonomy.rank.to_string());
            });

            query_builder.build().execute(&mut tx).await?;
        }
    }

    // taxonomy relation
    {
        let relations: Vec<(String, String, String)> = taxonomies
            .iter()
            .map(|t| {
                t.children.iter().map(|c| {
                    (
                        t.parent_id.to_string(),
                        c.id.to_string(),
                        c.rank.to_string(),
                    )
                })
            })
            .flatten()
            .collect();

        println!("relations number: {}", relations.len());

        for chunk in relations.chunks(SQLITE_LIMIT / 3) {
            let mut query_builder: QueryBuilder<Sqlite> =
                QueryBuilder::new("INSERT INTO taxonomy_relation(parent_id, child_id, rank) ");

            query_builder.push_values(chunk, |mut b, (parent_id, child_id, rank)| {
                b.push_bind(parent_id).push_bind(child_id).push_bind(rank);
            });

            query_builder.build().execute(&mut tx).await?;
        }
    }

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod test_sql {
    use std::path::PathBuf;

    use anyhow::Result;

    use crate::{downlaod_taxonomy_dump, TaxonomyRecord, _load_taxonomy_from_dump};

    use super::{insert_taxonomies, setup_db};

    #[sqlx::test]
    async fn test_create_table() -> Result<()> {
        let pool = setup_db().await?;

        println!("Downloading data...");
        let tmp = PathBuf::from("./_tmp");
        tokio::task::spawn_blocking(move || {
            downlaod_taxonomy_dump(tmp.as_path()).expect("Error in download and taxonomy dump");
        })
        .await?;

        println!("Parse raw dump files...");
        let recs = _load_taxonomy_from_dump(
            &PathBuf::from("./_tmp/nodes.dmp"),
            &PathBuf::from("./_tmp/names.dmp"),
        )?;
        println!("taxonomy_num: {}", recs.len());

        let recs: Vec<TaxonomyRecord> = recs.into_values().collect();

        println!("Insert to sqlite3");
        insert_taxonomies(&pool, &recs).await?;
        Ok(())
    }

    // #[sqlx::test]
    // async fn test_retrive() -> Result<()> {
    //     let pool = setup_db().await?;
    //     let mut conn = pool.acquire().await?;
    //     struct Taxonomy {
    //         id: Option<String>,
    //         name: Option<String>,
    //         rank: Option<String>,
    //     }
    //     let val = sqlx::query_as!(Taxonomy, "SELECT * FROM taxonomy_record WHERE id = 332963").fetch_one(&mut conn).await?;

    //     Ok(())
    // }
}
