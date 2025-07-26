use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use serenity::all::{GuildId, GuildPagination, Http, Member, Permissions, RoleId, UserId};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::repository::{ManagerRole, SettingRepository};

const UPDATE_PERIOD: Duration = Duration::from_secs(2 * 3600);

pub struct PermissionService {
    http: Arc<Http>,
    setting_repository: Arc<SettingRepository>,
    admin_roles: Arc<RwLock<HashMap<GuildId, HashSet<RoleId>>>>,
    guild_owners: Arc<RwLock<HashMap<GuildId, UserId>>>,
}

impl PermissionService {
    pub fn new(http: Arc<Http>, setting_repository: Arc<SettingRepository>) -> PermissionService {
        PermissionService {
            http,
            setting_repository,
            admin_roles: Arc::new(RwLock::new(HashMap::new())),
            guild_owners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_permissions(
        &self,
        guild_id: GuildId,
        member: &Member,
    ) -> anyhow::Result<bool> {
        info!(
            "Checking permissions of member {} in guild {}",
            member.user.id, guild_id
        );

        match self.setting_repository.get::<ManagerRole>(guild_id).await {
            Ok(Some(manager_role)) => {
                if member.roles.iter().any(|r| *r == manager_role.role) {
                    debug!("Permission check passed - user is a manager");
                    return Ok(true);
                }
            }
            _ => {
                debug!("Manager role is not configured");
            }
        };

        {
            let guard = self.admin_roles.read().await;
            if let Some(roles) = guard.get(&guild_id) {
                if member.roles.iter().any(|role| roles.contains(&role)) {
                    debug!("Permission check passed - user is an admin");
                    return Ok(true);
                }
            }
        }

        {
            let guard = self.guild_owners.read().await;
            if let Some(owner) = guard.get(&guild_id) {
                if *owner == member.user.id {
                    debug!("Permission check passed - user is the guild owner");
                    return Ok(true);
                }
            }
        }

        debug!("Permission check failed");
        Ok(false)
    }

    pub fn start(&self) {
        let http = self.http.clone();
        let admin_roles = self.admin_roles.clone();
        let guild_owners = self.guild_owners.clone();

        tokio::spawn(async move {
            loop {
                info!("Updating permission cache");

                if let Err(err) = Self::update_cache(&http, &admin_roles, &guild_owners).await {
                    warn!("Could not update permission cache: {err}");
                }

                info!("Sleeping for {UPDATE_PERIOD:?} until next update");
                tokio::time::sleep(UPDATE_PERIOD).await;
            }
        });
    }

    async fn update_cache(
        http: &Http,
        admin_roles: &RwLock<HashMap<GuildId, HashSet<RoleId>>>,
        guild_owners: &RwLock<HashMap<GuildId, UserId>>,
    ) -> anyhow::Result<()> {
        let guilds = {
            let per_page = 100;
            let mut last_guild = None;
            let mut guilds = Vec::new();

            loop {
                let page = http
                    .get_guilds(
                        last_guild.map(|g| GuildPagination::After(g)),
                        Some(per_page),
                    )
                    .await?;
                for guild_info in &page {
                    guilds.push(guild_info.id);
                }

                if page.len() < per_page as _ {
                    break;
                } else {
                    last_guild = Some(
                        page.last()
                            .expect("Page must be non-empty because of the outer check")
                            .id,
                    );
                }
            }

            guilds
        };

        let mut new_admin_roles = HashMap::new();
        let mut new_guild_owners = HashMap::new();

        for guild_id in guilds {
            debug!("Getting roles for guild {guild_id}");
            let roles = http.get_guild_roles(guild_id).await?;
            new_admin_roles.insert(
                guild_id,
                roles
                    .iter()
                    .filter(|role| role.has_permission(Permissions::ADMINISTRATOR))
                    .map(|role| role.id)
                    .collect::<HashSet<RoleId>>(),
            );

            debug!("Getting owner of guild {guild_id}");
            let guild = http.get_guild(guild_id).await?;
            new_guild_owners.insert(guild_id, guild.owner_id);
        }

        {
            let mut guard = admin_roles.write().await;
            *guard = new_admin_roles;
        }

        {
            let mut guard = guild_owners.write().await;
            *guard = new_guild_owners;
        }

        Ok(())
    }
}
