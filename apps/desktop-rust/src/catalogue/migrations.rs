use rusqlite::Connection;

use super::CatalogueError;

const MIGRATIONS: [&str; 8] = [
    include_str!(
        "../../../desktop/electron/infrastructure/sqlite/migrations/0001-create-libraries.sql"
    ),
    include_str!(
        "../../../desktop/electron/infrastructure/sqlite/migrations/0002-create-tracks.sql"
    ),
    include_str!(
        "../../../desktop/electron/infrastructure/sqlite/migrations/0003-track-metadata-overrides.sql"
    ),
    include_str!(
        "../../../desktop/electron/infrastructure/sqlite/migrations/0004-create-playlists.sql"
    ),
    include_str!(
        "../../../desktop/electron/infrastructure/sqlite/migrations/0005-create-play-history.sql"
    ),
    include_str!(
        "../../../desktop/electron/infrastructure/sqlite/migrations/0006-create-dropbox-accounts.sql"
    ),
    include_str!(
        "../../../desktop/electron/infrastructure/sqlite/migrations/0007-dropbox-client-id.sql"
    ),
    include_str!(
        "../../../desktop/electron/infrastructure/sqlite/migrations/0008-dropbox-library-sync.sql"
    ),
];

pub fn run(database: &mut Connection) -> Result<(), CatalogueError> {
    let transaction = database.transaction()?;
    let reported: u32 = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if reported as usize > MIGRATIONS.len() {
        return Err(CatalogueError::UnsupportedSchema);
    }
    let version = if reported == 0 {
        infer_version(&transaction)?
    } else {
        reported as usize
    };

    for (index, sql) in MIGRATIONS.iter().enumerate().skip(version) {
        transaction.execute_batch(sql)?;
        transaction.pragma_update(None, "user_version", (index + 1) as u32)?;
    }
    transaction.commit()?;
    Ok(())
}

fn infer_version(database: &Connection) -> Result<usize, CatalogueError> {
    const TABLES: [&str; 5] = [
        "libraries",
        "tracks",
        "playlists",
        "playlist_entries",
        "play_history",
    ];
    let observed = TABLES
        .map(|name| columns(database, name))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    if observed.iter().all(Vec::is_empty) {
        return Ok(0);
    }
    let expected = Connection::open_in_memory()?;
    for (index, sql) in MIGRATIONS.iter().take(TABLES.len()).enumerate() {
        expected.execute_batch(sql)?;
        let signatures = TABLES
            .map(|name| columns(&expected, name))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        if signatures == observed {
            return Ok(index + 1);
        }
    }
    Err(CatalogueError::UnsupportedSchema)
}

fn columns(
    database: &Connection,
    table: &str,
) -> rusqlite::Result<Vec<(String, String, bool, bool)>> {
    let mut statement = database
        .prepare("SELECT name, type, \"notnull\", pk FROM pragma_table_info(?1) ORDER BY cid")?;
    statement
        .query_map([table], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect()
}
