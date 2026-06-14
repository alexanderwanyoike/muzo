use muzo_desktop_lib::domain::library::{
    Library, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
};
use muzo_desktop_lib::infrastructure::sqlite_library_repository::SqliteLibraryRepository;

fn make_library(name: &str, location: &str) -> Library {
    Library::new(
        muzo_desktop_lib::domain::library::generate_library_id(),
        LibraryName(name.into()),
        LibraryKind::Filesystem,
        LibraryLocation(location.into()),
    )
}

#[test]
fn a_library_added_to_sqlite_can_be_found_by_its_id() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    SqliteLibraryRepository::migrate(&conn).unwrap();
    let repo = SqliteLibraryRepository::new(conn);

    let library = make_library("My Music", "/home/user/Music");
    repo.add(&library).unwrap();

    let found = repo
        .find_by_id(library.id())
        .expect("query should not error")
        .expect("library should be found by id");

    assert_eq!(found, library);
}

#[test]
fn find_by_id_returns_none_when_the_library_is_not_there() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    SqliteLibraryRepository::migrate(&conn).unwrap();
    let repo = SqliteLibraryRepository::new(conn);

    use muzo_desktop_lib::domain::library::LibraryId;
    let found = repo
        .find_by_id(&LibraryId("does-not-exist".into()))
        .expect("query should not error");

    assert!(found.is_none());
}

#[test]
fn adding_two_libraries_with_distinct_ids_stores_both() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    SqliteLibraryRepository::migrate(&conn).unwrap();
    let repo = SqliteLibraryRepository::new(conn);

    let a = make_library("First", "/a");
    let b = make_library("Second", "/b");
    repo.add(&a).unwrap();
    repo.add(&b).unwrap();

    assert_eq!(
        repo.find_by_id(a.id()).unwrap().unwrap(),
        a,
        "first library survives the second add"
    );
    assert_eq!(
        repo.find_by_id(b.id()).unwrap().unwrap(),
        b,
        "second library is retrievable"
    );
}

#[test]
fn list_returns_every_persisted_library_in_insertion_order() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    SqliteLibraryRepository::migrate(&conn).unwrap();
    let repo = SqliteLibraryRepository::new(conn);

    let a = make_library("First", "/a");
    let b = make_library("Second", "/b");
    repo.add(&a).unwrap();
    repo.add(&b).unwrap();

    let listed = repo.list().expect("list should succeed");

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0], a, "first added comes back first");
    assert_eq!(listed[1], b, "second added comes back second");
}

#[test]
fn list_returns_an_empty_vec_when_no_libraries_have_been_added() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    SqliteLibraryRepository::migrate(&conn).unwrap();
    let repo = SqliteLibraryRepository::new(conn);

    let listed = repo.list().expect("list should succeed");

    assert!(
        listed.is_empty(),
        "fresh database should yield an empty list"
    );
}
