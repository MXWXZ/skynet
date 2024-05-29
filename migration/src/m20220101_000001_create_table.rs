use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum Users {
    Table,
    ID,
    Username,
    Password,
    Avatar,
    LastLogin,
    LastIP,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Groups {
    Table,
    ID,
    Name,
    Note,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum UserGroupLinks {
    Table,
    ID,
    Uid,
    Gid,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PermissionLinks {
    Table,
    ID,
    Uid,
    Gid,
    Pid,
    Perm,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Permissions {
    Table,
    ID,
    Name,
    Note,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Settings {
    Table,
    ID,
    Name,
    Value,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Notifications {
    Table,
    ID,
    Level,
    Target,
    Message,
    Detail,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::ID)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string_len(32)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Users::Password).string_len(128).not_null())
                    .col(ColumnDef::new(Users::Avatar).binary())
                    .col(ColumnDef::new(Users::LastLogin).big_integer())
                    .col(ColumnDef::new(Users::LastIP).string_len(64))
                    .col(ColumnDef::new(Users::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).big_integer().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Groups::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Groups::ID)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Groups::Name)
                            .string_len(32)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Groups::Note).string_len(256).not_null())
                    .col(ColumnDef::new(Groups::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Groups::UpdatedAt).big_integer().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(UserGroupLinks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserGroupLinks::ID)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserGroupLinks::Uid).char_len(36).not_null())
                    .col(ColumnDef::new(UserGroupLinks::Gid).char_len(36).not_null())
                    .col(
                        ColumnDef::new(UserGroupLinks::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserGroupLinks::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .to(Users::Table, Users::ID)
                            .from_col(UserGroupLinks::Uid)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .to(Groups::Table, Groups::ID)
                            .from_col(UserGroupLinks::Gid)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_usergrouplinks_1")
                    .table(UserGroupLinks::Table)
                    .col(UserGroupLinks::Uid)
                    .col(UserGroupLinks::Gid)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Permissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Permissions::ID)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Permissions::Name)
                            .string_len(128)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Permissions::Note).string_len(256).not_null())
                    .col(
                        ColumnDef::new(Permissions::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Permissions::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(PermissionLinks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PermissionLinks::ID)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PermissionLinks::Uid).char_len(36))
                    .col(ColumnDef::new(PermissionLinks::Gid).char_len(36))
                    .col(ColumnDef::new(PermissionLinks::Pid).char_len(36).not_null())
                    .col(
                        ColumnDef::new(PermissionLinks::Perm)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PermissionLinks::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PermissionLinks::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .to(Users::Table, Users::ID)
                            .from_col(PermissionLinks::Uid)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .to(Groups::Table, Groups::ID)
                            .from_col(PermissionLinks::Gid)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .to(Permissions::Table, Permissions::ID)
                            .from_col(PermissionLinks::Pid)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_permissionlinks_1")
                    .table(PermissionLinks::Table)
                    .col(PermissionLinks::Uid)
                    .col(PermissionLinks::Gid)
                    .col(PermissionLinks::Pid)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Settings::ID)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Settings::Name)
                            .string_len(256)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Settings::Value).string().not_null())
                    .col(ColumnDef::new(Settings::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Settings::UpdatedAt).big_integer().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Notifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Notifications::ID)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Notifications::Level)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Notifications::Target)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Notifications::Message).string().not_null())
                    .col(ColumnDef::new(Notifications::Detail).string().not_null())
                    .col(
                        ColumnDef::new(Notifications::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Notifications::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Groups::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserGroupLinks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Permissions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PermissionLinks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Notifications::Table).to_owned())
            .await?;
        Ok(())
    }
}
