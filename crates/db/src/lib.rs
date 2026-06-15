pub mod database;

pub use database::Database;

use redb::TableDefinition;

pub enum ContentLocation {
    Disk(String),
    Database,
    Remote(String),
}

pub const FILE_CONTENTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("file_contents");
