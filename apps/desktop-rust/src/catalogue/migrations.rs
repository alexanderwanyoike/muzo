use rusqlite::Connection;

use super::CatalogueError;

const MIGRATIONS: [&str; 9] = [
    include_str!("../../migrations/0001-create-libraries.sql"),
    include_str!("../../migrations/0002-create-tracks.sql"),
    include_str!("../../migrations/0003-track-metadata-overrides.sql"),
    include_str!("../../migrations/0004-create-playlists.sql"),
    include_str!("../../migrations/0005-create-play-history.sql"),
    include_str!("../../migrations/0006-create-dropbox-accounts.sql"),
    include_str!("../../migrations/0007-dropbox-client-id.sql"),
    include_str!("../../migrations/0008-dropbox-library-sync.sql"),
    "ALTER TABLE dropbox_accounts ADD COLUMN credential_store TEXT
     CHECK (credential_store IN ('keyring', 'kwallet'));",
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
        let version = reported as usize;
        validate_version(&transaction, version)?;
        version
    };

    for (index, sql) in MIGRATIONS.iter().enumerate().skip(version) {
        transaction.execute_batch(sql)?;
        transaction.pragma_update(None, "user_version", (index + 1) as u32)?;
    }
    validate_version(&transaction, MIGRATIONS.len())?;
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

fn validate_version(database: &Connection, version: usize) -> Result<(), CatalogueError> {
    const TABLES: [&str; 8] = [
        "libraries",
        "tracks",
        "playlists",
        "playlist_entries",
        "play_history",
        "dropbox_accounts",
        "dropbox_library_accounts",
        "dropbox_sync_settings",
    ];
    let expected = Connection::open_in_memory()?;
    for sql in MIGRATIONS.iter().take(version) {
        expected.execute_batch(sql)?;
    }
    for table in TABLES {
        if columns(database, table)? != columns(&expected, table)?
            || unique_keys(database, table)? != unique_keys(&expected, table)?
        {
            return Err(CatalogueError::UnsupportedSchema);
        }
    }
    Ok(())
}

fn unique_keys(database: &Connection, table: &str) -> rusqlite::Result<Vec<String>> {
    let mut statement = database.prepare(
        "SELECT group_concat(name, ',') FROM (
            SELECT l.name AS index_name, i.name AS name
            FROM pragma_index_list(?1) l JOIN pragma_index_info(l.name) i
            WHERE l.[unique]=1 ORDER BY l.name, i.seqno
        ) GROUP BY index_name ORDER BY 1",
    )?;
    statement.query_map([table], |row| row.get(0))?.collect()
}
