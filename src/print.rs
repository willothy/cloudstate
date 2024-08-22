use crate::extensions::cloudstate::OBJECTS_TABLE;
use redb::ReadableTable;

pub fn print_database(db: &redb::Database) {
    let txn = db.begin_read().unwrap();
    let table = txn.open_table(OBJECTS_TABLE).unwrap();

    for entry in table.iter().unwrap() {
        let entry = entry.unwrap();
        println!("{:?}: {:?}", entry.0.value().id, entry.1.value().data);
    }
}