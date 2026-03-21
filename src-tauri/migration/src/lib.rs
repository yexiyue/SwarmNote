pub use sea_orm_migration::prelude::*;

mod m20260321_000001_init_devices;
mod m20260321_000002_init_workspace;

pub struct DevicesMigrator;

#[async_trait::async_trait]
impl MigratorTrait for DevicesMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260321_000001_init_devices::Migration)]
    }
}

pub struct WorkspaceMigrator;

#[async_trait::async_trait]
impl MigratorTrait for WorkspaceMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260321_000002_init_workspace::Migration)]
    }
}
