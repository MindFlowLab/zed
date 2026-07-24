//! 会话恢复持久化：记录预览标签页对应的文档路径，
//! 重启后按路径重新解析打开。

use std::path::PathBuf;

use db::{
    query,
    sqlez::{domain::Domain, thread_safe_connection::ThreadSafeConnection},
    sqlez_macros::sql,
};
use workspace::{ItemId, WorkspaceDb, WorkspaceId};

pub struct OfficePreviewDb(ThreadSafeConnection);

impl Domain for OfficePreviewDb {
    const NAME: &str = stringify!(OfficePreviewDb);

    const MIGRATIONS: &[&str] = &[sql!(
            CREATE TABLE office_previews (
                workspace_id INTEGER,
                item_id INTEGER UNIQUE,

                document_path BLOB,

                PRIMARY KEY(workspace_id, item_id),
                FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                ON DELETE CASCADE
            ) STRICT;
    )];
}

db::static_connection!(OfficePreviewDb, [WorkspaceDb]);

impl OfficePreviewDb {
    query! {
        pub async fn save_document_path(
            item_id: ItemId,
            workspace_id: WorkspaceId,
            document_path: PathBuf
        ) -> Result<()> {
            INSERT OR REPLACE INTO office_previews(item_id, workspace_id, document_path)
            VALUES (?, ?, ?)
        }
    }

    query! {
        pub fn get_document_path(item_id: ItemId, workspace_id: WorkspaceId) -> Result<Option<PathBuf>> {
            SELECT document_path
            FROM office_previews
            WHERE item_id = ? AND workspace_id = ?
        }
    }
}
