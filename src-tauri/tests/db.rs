use entity::workspace::{documents, folders, workspaces};
use migration::{DevicesMigrator, MigratorTrait, WorkspaceMigrator};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};

async fn setup_devices_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    DevicesMigrator::up(&db, None).await.unwrap();
    db
}

async fn setup_workspace_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    WorkspaceMigrator::up(&db, None).await.unwrap();
    db
}

#[tokio::test]
async fn devices_db_creates_paired_devices_table() {
    let db = setup_devices_db().await;
    let count = entity::devices::paired_devices::Entity::find()
        .count(&db)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn workspace_db_creates_all_seven_tables() {
    let db = setup_workspace_db().await;

    assert_eq!(workspaces::Entity::find().count(&db).await.unwrap(), 0);
    assert_eq!(folders::Entity::find().count(&db).await.unwrap(), 0);
    assert_eq!(documents::Entity::find().count(&db).await.unwrap(), 0);
    assert_eq!(
        entity::workspace::workspace_keys::Entity::find()
            .count(&db)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        entity::workspace::doc_chunks::Entity::find()
            .count(&db)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        entity::workspace::permissions::Entity::find()
            .count(&db)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        entity::workspace::share_invites::Entity::find()
            .count(&db)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn workspace_insert_and_find_by_id() {
    let db = setup_workspace_db().await;

    let ws = workspaces::ActiveModel {
        id: Set("ws-001".to_string()),
        name: Set("Test Workspace".to_string()),
        created_by: Set("peer-abc".to_string()),
        created_at: Set(1000),
        updated_at: Set(1000),
        ..Default::default()
    };
    let ws = ws.insert(&db).await.unwrap();
    assert_eq!(ws.name, "Test Workspace");

    let found = workspaces::Entity::find_by_id("ws-001")
        .one(&db)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test Workspace");
}

#[tokio::test]
async fn document_insert_update_delete() {
    let db = setup_workspace_db().await;

    workspaces::ActiveModel {
        id: Set("ws-001".to_string()),
        name: Set("Test".to_string()),
        created_by: Set("peer-abc".to_string()),
        created_at: Set(1000),
        updated_at: Set(1000),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    // Insert
    let doc = documents::ActiveModel {
        id: Set("doc-001".to_string()),
        workspace_id: Set("ws-001".to_string()),
        folder_id: Set(None),
        title: Set("My Note".to_string()),
        rel_path: Set("my-note.md".to_string()),
        file_hash: Set(None),
        yjs_state: Set(None),
        state_vector: Set(None),
        created_by: Set("peer-abc".to_string()),
        created_at: Set(1000),
        updated_at: Set(1000),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();
    assert_eq!(doc.title, "My Note");

    // Query
    let docs = documents::Entity::find()
        .filter(documents::Column::WorkspaceId.eq("ws-001"))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(docs.len(), 1);

    // Update
    let mut active: documents::ActiveModel = doc.into();
    active.title = Set("Updated Note".to_string());
    active.updated_at = Set(2000);
    let updated = active.update(&db).await.unwrap();
    assert_eq!(updated.title, "Updated Note");
    assert_eq!(updated.updated_at, 2000);

    // Delete
    documents::Entity::delete_by_id("doc-001")
        .exec(&db)
        .await
        .unwrap();
    assert_eq!(documents::Entity::find().count(&db).await.unwrap(), 0);
}

#[tokio::test]
async fn folder_hierarchy_parent_child() {
    let db = setup_workspace_db().await;

    workspaces::ActiveModel {
        id: Set("ws-001".to_string()),
        name: Set("Test".to_string()),
        created_by: Set("peer-abc".to_string()),
        created_at: Set(1000),
        updated_at: Set(1000),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    // Root folder
    folders::ActiveModel {
        id: Set("folder-root".to_string()),
        workspace_id: Set("ws-001".to_string()),
        parent_folder_id: Set(None),
        name: Set("Notes".to_string()),
        rel_path: Set("Notes".to_string()),
        created_by: Set("peer-abc".to_string()),
        created_at: Set(1000),
        updated_at: Set(1000),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    // Child folder
    folders::ActiveModel {
        id: Set("folder-child".to_string()),
        workspace_id: Set("ws-001".to_string()),
        parent_folder_id: Set(Some("folder-root".to_string())),
        name: Set("Daily".to_string()),
        rel_path: Set("Notes/Daily".to_string()),
        created_by: Set("peer-abc".to_string()),
        created_at: Set(1000),
        updated_at: Set(1000),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    let all = folders::Entity::find()
        .filter(folders::Column::WorkspaceId.eq("ws-001"))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(all.len(), 2);

    let children = folders::Entity::find()
        .filter(folders::Column::ParentFolderId.eq(Some("folder-root".to_string())))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name, "Daily");
}

#[tokio::test]
async fn migration_is_idempotent() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    WorkspaceMigrator::up(&db, None).await.unwrap();
    WorkspaceMigrator::up(&db, None).await.unwrap();

    assert_eq!(workspaces::Entity::find().count(&db).await.unwrap(), 0);
}
