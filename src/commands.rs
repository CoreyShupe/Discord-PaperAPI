use chrono;
use paper_api::paper::BuildDownloadRequest;
use paper_api::{PaperClient, PaperClientConfig, BASE_URL};
use serenity::builder::CreateEmbedAuthor;
use serenity::client::Context;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::channel::Message;
use serenity::model::user::User;

const COMMAND_COLOR: (u8, u8, u8) = (56, 210, 237);

fn def_embed_author(user: &User) -> CreateEmbedAuthor {
    let mut creator = CreateEmbedAuthor::default();
    if let Some(url) = &user.avatar_url() {
        &creator.icon_url(url);
    }
    &creator.name(&user.name);
    creator
}

fn timestamp() -> String {
    chrono::offset::Utc::now().to_rfc3339()
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

#[group]
#[commands(projects, project)]
pub struct General;

#[command]
async fn projects(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.author.bot {
        msg.reply(ctx, "Cannot respond to bots.").await?;
    }

    let projects = PaperClient::get_projects().await?;
    msg.channel_id
        .send_message(ctx, |message_builder| {
            message_builder
                .reference_message(msg)
                .allowed_mentions(|f| f.replied_user(true));
            message_builder.add_embed(|embed_builder| {
                embed_builder.color(COMMAND_COLOR);
                embed_builder.set_author(def_embed_author(&msg.author));
                embed_builder.title("PaperAPI Projects");
                embed_builder.description(projects.projects.join(", "));
                embed_builder.timestamp(timestamp());
                embed_builder
            });

            message_builder
        })
        .await?;

    Ok(())
}

#[command]
#[sub_commands(project_groups, project_builds, project_version, project_build)]
async fn project(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.author.bot {
        msg.reply(ctx, "Cannot respond to bots.").await?;
    }

    let project_name = if let Ok(project_name) = args.single::<String>() {
        project_name
    } else {
        msg.reply(
            ctx,
            "Did not understand request. Please use `>project <project name>`",
        )
        .await?;
        return Ok(());
    };

    if let Ok(project) = PaperClient::get_project(&project_name).await {
        let mut project_groups: Vec<String> = project.version_groups.clone();
        let mut project_versions: Vec<String> = project.versions.clone();

        msg.channel_id
            .send_message(ctx, |message_builder| {
                project_groups.reverse();
                project_versions.reverse();

                message_builder
                    .reference_message(msg)
                    .allowed_mentions(|f| f.replied_user(true));
                message_builder.add_embed(|embed_builder| {
                    embed_builder
                        .color(COMMAND_COLOR)
                        .set_author(def_embed_author(&msg.author))
                        .title(format!("PaperAPI Project {}", project.project_name))
                        .field("Project ID", project.project_id, true)
                        .field(
                            "Project Groups",
                            truncate(&project_groups.join(", "), 1024),
                            false,
                        )
                        .field(
                            "Project Versions",
                            truncate(&project_versions.join(", "), 1024),
                            false,
                        )
                        .timestamp(timestamp())
                });

                message_builder
            })
            .await?;
    } else {
        msg.reply(ctx, format!("Could not find project {}", &project_name))
            .await?;
    }

    Ok(())
}

#[command("groups")]
async fn project_groups(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let project_name = if let Ok(project_name) = args.single::<String>() {
        project_name
    } else {
        msg.reply(ctx, "Did not understand request. Please use `>project groups <project name> <version group>`").await?;
        return Ok(());
    };

    let version_group = if let Ok(version_group) = args.single::<String>() {
        version_group
    } else {
        msg.reply(ctx, "Did not understand request. Please use `>project groups <project name> <version group>`").await?;
        return Ok(());
    };

    if let Ok(group_info) = PaperClient::get_group_info(&project_name, &version_group).await {
        let mut project_versions: Vec<String> = group_info.versions.clone();

        msg.channel_id
            .send_message(ctx, |message_builder| {
                project_versions.reverse();

                message_builder
                    .reference_message(msg)
                    .allowed_mentions(|f| f.replied_user(true));
                message_builder.add_embed(|embed_builder| {
                    embed_builder
                        .color(COMMAND_COLOR)
                        .set_author(def_embed_author(&msg.author))
                        .title(format!(
                            "PaperAPI Project {} Group {}",
                            group_info.project_name, group_info.version_group
                        ))
                        .field("Project ID", group_info.project_id, true)
                        .field(
                            "Project Versions",
                            truncate(&project_versions.join(", "), 1024),
                            false,
                        )
                        .timestamp(timestamp())
                });

                message_builder
            })
            .await?;
    } else {
        msg.reply(
            ctx,
            format!(
                "Could not find project groups (Project: {}, Version Group: {})",
                &project_name, &version_group
            ),
        )
        .await?;
    }

    Ok(())
}

#[command("builds")]
async fn project_builds(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let project_name = if let Ok(project_name) = args.single::<String>() {
        project_name
    } else {
        msg.reply(ctx, "Did not understand request. Please use `>project builds <project name> <version group>`").await?;
        return Ok(());
    };

    let version_group = if let Ok(version_group) = args.single::<String>() {
        version_group
    } else {
        msg.reply(ctx, "Did not understand request. Please use `>project builds <project name> <version group>`").await?;
        return Ok(());
    };

    if let Ok(builds_info) = PaperClient::get_group_builds(&project_name, &version_group).await {
        let mut project_versions: Vec<String> = builds_info.versions.clone();
        let mut group_builds: Vec<String> = builds_info
            .builds
            .iter()
            .map(|info| format!("{}", info.build))
            .collect::<Vec<String>>();

        msg.channel_id
            .send_message(ctx, |message_builder| {
                project_versions.reverse();
                group_builds.reverse();

                message_builder
                    .reference_message(msg)
                    .allowed_mentions(|f| f.replied_user(true));
                message_builder.add_embed(|embed_builder| {
                    embed_builder
                        .color(COMMAND_COLOR)
                        .set_author(def_embed_author(&msg.author))
                        .title(format!(
                            "PaperAPI Project {} Group {}",
                            builds_info.project_name, builds_info.version_group
                        ))
                        .field("Project ID", builds_info.project_id, true)
                        .field(
                            "Project Versions",
                            truncate(&project_versions.join(", "), 1024),
                            false,
                        )
                        .field(
                            "Project Version Group Builds",
                            truncate(&group_builds.join(", "), 1024),
                            false,
                        )
                        .timestamp(timestamp())
                });

                message_builder
            })
            .await?;
    } else {
        msg.reply(
            ctx,
            format!(
                "Could not find builds (Project: {}, Version Group: {})",
                &project_name, &version_group
            ),
        )
        .await?;
    }

    Ok(())
}

#[command("version")]
async fn project_version(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let project_name = if let Ok(project_name) = args.single::<String>() {
        project_name
    } else {
        msg.reply(
            ctx,
            "Did not understand request. Please use `>project version <project name> <version>`",
        )
        .await?;
        return Ok(());
    };

    let version = if let Ok(version) = args.single::<String>() {
        version
    } else {
        msg.reply(
            ctx,
            "Did not understand request. Please use `>project version <project name> <version>`",
        )
        .await?;
        return Ok(());
    };

    if let Ok(version_info) = PaperClient::get_version_info(&project_name, &version).await {
        let mut version_builds: Vec<String> = version_info
            .builds
            .iter()
            .map(|info| format!("{}", info))
            .collect::<Vec<String>>();

        msg.channel_id
            .send_message(ctx, |message_builder| {
                version_builds.reverse();

                message_builder
                    .reference_message(msg)
                    .allowed_mentions(|f| f.replied_user(true));
                message_builder.add_embed(|embed_builder| {
                    embed_builder
                        .color(COMMAND_COLOR)
                        .set_author(def_embed_author(&msg.author))
                        .title(format!(
                            "PaperAPI Project {} Version {}",
                            version_info.project_name, version_info.version
                        ))
                        .field("Project ID", version_info.project_id, true)
                        .field(
                            "Project Version Builds",
                            truncate(&version_builds.join(", "), 1024),
                            false,
                        )
                        .timestamp(timestamp())
                });

                message_builder
            })
            .await?;
    } else {
        msg.reply(
            ctx,
            format!(
                "Could not find project version (Project: {}, Version: {})",
                &project_name, &version
            ),
        )
        .await?;
    }

    Ok(())
}

#[command("build")]
async fn project_build(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let project_name = if let Ok(project_name) = args.single::<String>() {
        project_name
    } else {
        msg.reply(ctx, "Did not understand request. Please use `>project build <project name> <version> <build>`").await?;
        return Ok(());
    };

    let version = if let Ok(version) = args.single::<String>() {
        version
    } else {
        msg.reply(ctx, "Did not understand request. Please use `>project build <project name> <version> <build>`").await?;
        return Ok(());
    };

    let build = if let Ok(build) = args.single::<i32>() {
        build
    } else {
        msg.reply(ctx, "Did not understand request. Please use `>project build <project name> <version> <build>`").await?;
        return Ok(());
    };

    if let Ok(version_build_info) =
        PaperClient::get_version_builds(&project_name, &version, build).await
    {
        let changes: Vec<(String, String)> = version_build_info
            .changes
            .iter()
            .take(23)
            .map(|change| {
                (
                    format!("Commit {}", truncate(&change.commit, 10)),
                    format!("{}", truncate(&change.summary, 1024)),
                )
            })
            .collect();

        let download = BuildDownloadRequest::new(
            &version_build_info.project_id,
            &version_build_info.version,
            version_build_info.build,
            &version_build_info.downloads.application.name,
        )
        .build_request_url();

        msg.channel_id
            .send_message(ctx, |message_builder| {
                message_builder
                    .reference_message(msg)
                    .allowed_mentions(|f| f.replied_user(true));
                message_builder.add_embed(|embed_builder| {
                    embed_builder
                        .color(COMMAND_COLOR)
                        .set_author(def_embed_author(&msg.author))
                        .title(format!(
                            "PaperAPI Project {} Version {} Build {}",
                            version_build_info.project_name,
                            version_build_info.version,
                            version_build_info.build
                        ))
                        .field("Project ID", &version_build_info.project_id, true)
                        .field("Time", &version_build_info.time, true)
                        .timestamp(timestamp())
                        .footer(|creator| creator.text("Click to download!"))
                        .url(format!("{}{}", BASE_URL, &download));
                    for x in changes {
                        embed_builder.field(x.0, x.1, false);
                    }
                    embed_builder
                });

                message_builder
            })
            .await?;
    } else {
        msg.reply(
            ctx,
            format!(
                "Could not find project version build (Project: {}, Version: {}, Build: {})",
                &project_name, &version, &build
            ),
        )
        .await?;
    }

    Ok(())
}
