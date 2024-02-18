use rusqlite::{params, Connection, DatabaseName, Result};
extern crate dirs;
use crate::Password;
pub struct Database {
    conn: Connection,
}
impl Database {
    pub fn new(key: String) -> Result<Database> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = std::path::Path::new(&manifest_dir).join("logs/passwords");

        let conn = Connection::open(path)?;

        conn.pragma_update(Some(DatabaseName::Main), "KEY", key)?;
        let db = Database { conn };
        db.create_table()?;
        Ok(db)
    }

    pub fn create_table(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS passwords(
                    id INTEGER PRIMARY KEY,
                    title TEXT NOT NULL,
                    username TEXT NOT NULL,
                    password TEXT NOT NULL
                )
            ",
        )?;
        Ok(())
    }

    pub fn load(&self) -> Vec<Password> {
        let mut statement = self.conn.prepare("select * from passwords").unwrap();
        let items: Vec<Password> = statement
            .query_map([], |row| {
                let password = Password::new_with_id(
                    row.get("id").unwrap(),
                    row.get("title").unwrap(),
                    row.get("username").unwrap(),
                    row.get("password").unwrap(),
                );
                Ok(password)
            })
            .unwrap()
            .map(|i| i.unwrap())
            .collect();
        items
    }

    pub fn insert_password(&self, pw: &Password) {
        self.conn
            .execute(
                "INSERT INTO passwords(title, username, password) VALUES(?1,?2,?3)",
                params![pw.title, pw.username, pw.password],
            )
            .unwrap();
    }

    pub fn delete_pw(&self, id: usize) {
        self.conn
            .execute("DELETE FROM passwords WHERE id=?1", params![id])
            .unwrap();
    }
}
