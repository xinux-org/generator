use crate::prelude::*;
use anyhow::{Context, Result, anyhow};
use log::{debug, info};
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

static UNSTABLE: &str = "https://channels.nixos.org/nixos-unstable";

pub async fn download(mut version: &str, sourcedir: &str) -> Result<()> {
    let verurl = format!("https://channels.nixos.org/{}", version);
    debug!("Checking nixpkgs version");
    let resp = reqwest::get(&verurl).await?;
    let latestnixpkgsver = if resp.status().is_success() {
        resp.url()
            .path_segments()
            .context("No path segments found")?
            .next_back()
            .context("Last element not found")?
            .to_string()
    } else {
        let resp = reqwest::get(UNSTABLE).await?;
        if resp.status().is_success() {
            version = "unstable";
            resp.url()
                .path_segments()
                .context("No path segments found")?
                .next_back()
                .context("Last element not found")?
                .to_string()
        } else {
            return Err(anyhow!("Could not find latest nixpkgs version"));
        }
    };
    debug!("Latest nixpkgs version: {}", latestnixpkgsver);

    let latestpkgsver = latestnixpkgsver
        .strip_prefix("nixos-")
        .unwrap_or(&latestnixpkgsver);
    let latestpkgsver = latestpkgsver
        .strip_prefix("nixpkgs-")
        .unwrap_or(latestpkgsver);
    info!("latestnixpkgsver: {}", latestpkgsver);

    // Check if source directory exists
    let srcdir = Path::new(sourcedir);
    if !srcdir.exists() {
        // create source directory
        fs::create_dir_all(srcdir)?;
    }

    // Check if latest version is already downloaded
    if let Ok(prevver) = fs::read_to_string(format!("{}/nixpkgs.ver", sourcedir))
        && prevver == latestpkgsver
        && Path::new(&format!("{}/nixpkgs.db", sourcedir)).exists()
    {
        debug!("No new version of nixpkgs found");
        return Ok(());
    }

    let url = format!("https://channels.nixos.org/{}/packages.json.br", version);

    // Download file with reqwest blocking
    debug!("Downloading packages.json.br");
    let client = reqwest::Client::builder().brotli(true).build()?;
    let resp = client.get(url).send().await?;
    if resp.status().is_success() {
        // resp is pkgsjson
        debug!("Successfully downloaded packages.json.br");
        let db = format!("sqlite://{}/nixpkgs.db", sourcedir);

        if Path::new(&format!("{}/nixpkgs.db", sourcedir)).exists() {
            fs::remove_file(format!("{}/nixpkgs.db", sourcedir))?;
        }

        debug!("Creating SQLite database");
        Sqlite::create_database(&db).await?;
        let pool = SqlitePool::connect(&db).await?;

        sqlx::query(
            r#"
                CREATE TABLE "pkgs" (
                    "attribute"	TEXT NOT NULL UNIQUE,
                    "system"	TEXT,
                    "pname"	TEXT,
                    "version"	TEXT,
                    PRIMARY KEY("attribute")
                )
                "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE "meta" (
                "attribute"	TEXT NOT NULL UNIQUE,
                "broken"	INTEGER,
                "insecure"	INTEGER,
                "unsupported"	INTEGER,
                "unfree"	INTEGER,
                "description"	TEXT,
                "longdescription"	TEXT,
                "homepage"	TEXT,
                "maintainers"	JSON,
                "position"	TEXT,
                "license"	JSON,
                "platforms"	JSON,
                FOREIGN KEY("attribute") REFERENCES "pkgs"("attribute"),
                PRIMARY KEY("attribute")
            )
                "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE UNIQUE INDEX "attributes" ON "pkgs" ("attribute")
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE UNIQUE INDEX "metaattributes" ON "meta" ("attribute")
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX "pnames" ON "pkgs" ("pname")
            "#,
        )
        .execute(&pool)
        .await?;

        debug!("Reading packages.json.br");

        let pkgjson = resp
            .json::<NixosPkgList>() // ::<NixosPkgList>()
            .await
            .expect("Failed to parse request into string");

        debug!("Creating csv data");
        let mut wtr = csv::Writer::from_writer(vec![]);
        for (pkg, data) in &pkgjson.packages {
            wtr.serialize((
                pkg,
                data.system.to_string(),
                data.pname.clone().unwrap_or_default(),
                data.version.clone().unwrap_or_default(),
            ))?;
        }

        let data = String::from_utf8(wtr.into_inner()?)?;
        debug!("Inserting data into database");

        let mut cmd = Command::new("sqlite3")
            .arg("-csv")
            .arg(format!("{}/nixpkgs.db", sourcedir))
            .arg(".import '|cat -' pkgs")
            .stdin(Stdio::piped())
            .spawn()?;

        let cmd_stdin = cmd.stdin.as_mut().unwrap();
        cmd_stdin.write_all(data.as_bytes())?;
        let _status = cmd.wait()?;
        let mut metawtr = csv::Writer::from_writer(vec![]);
        for (pkg, data) in &pkgjson.packages {
            metawtr.serialize((
                pkg,
                if let Some(x) = data.meta.broken {
                    if x { 1 } else { 0 }
                } else {
                    0
                },
                if let Some(x) = data.meta.insecure {
                    if x { 1 } else { 0 }
                } else {
                    0
                },
                if let Some(x) = data.meta.unsupported {
                    if x { 1 } else { 0 }
                } else {
                    0
                },
                if let Some(x) = data.meta.unfree {
                    if x { 1 } else { 0 }
                } else {
                    0
                },
                data.meta.description.as_ref().map(|x| x.to_string()),
                data.meta.longdescription.as_ref().map(|x| x.to_string()),
                data.meta.homepage.as_ref().and_then(|x| match x {
                    StrOrVec::List(x) => x.first().map(|x| x.to_string()),
                    StrOrVec::Single(x) => Some(x.to_string()),
                }),
                data.meta
                    .maintainers
                    .as_ref()
                    .and_then(|x| serde_json::to_string(x).ok()),
                data.meta.position.as_ref().map(|x| x.to_string()),
                data.meta
                    .license
                    .as_ref()
                    .and_then(|x| serde_json::to_string(x).ok()),
                data.meta.platforms.as_ref().and_then(|x| match x {
                    Platform::Unknown(_) => None,
                    _ => serde_json::to_string(x).ok(),
                }),
            ))?;
        }
        let metadata = String::from_utf8(metawtr.into_inner()?)?;
        debug!("Inserting metadata into database");
        let mut metacmd = Command::new("sqlite3")
            .arg("-csv")
            .arg(format!("{}/nixpkgs.db", sourcedir))
            .arg(".import '|cat -' meta")
            .stdin(Stdio::piped())
            .spawn()?;
        let metacmd_stdin = metacmd.stdin.as_mut().unwrap();
        metacmd_stdin.write_all(metadata.as_bytes())?;
        let _status = metacmd.wait()?;
        debug!("Finished creating nixpkgs database");

        // Create version database
        let db = format!("sqlite://{}/nixpkgs_versions.db", sourcedir);
        Sqlite::create_database(&db).await?;
        let pool = SqlitePool::connect(&db).await?;
        sqlx::query(
            r#"
                CREATE TABLE "pkgs" (
                    "attribute"	TEXT NOT NULL UNIQUE,
                    "pname"	TEXT,
                    "version"	TEXT,
                    PRIMARY KEY("attribute")
                )
                "#,
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            r#"
            CREATE UNIQUE INDEX "attributes" ON "pkgs" ("attribute")
            "#,
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            r#"
            CREATE INDEX "pnames" ON "pkgs" ("attribute")
            "#,
        )
        .execute(&pool)
        .await?;

        let mut wtr = csv::Writer::from_writer(vec![]);
        for (pkg, data) in &pkgjson.packages {
            wtr.serialize((
                pkg,
                data.pname.clone().unwrap_or_default(),
                data.version.clone().unwrap_or_default(),
            ))?;
        }
        let data = String::from_utf8(wtr.into_inner()?)?;
        let mut cmd = Command::new("sqlite3")
            .arg("-csv")
            .arg(format!("{}/nixpkgs_versions.db", sourcedir))
            .arg(".import '|cat -' pkgs")
            .stdin(Stdio::piped())
            .spawn()?;
        let cmd_stdin = cmd.stdin.as_mut().unwrap();
        cmd_stdin.write_all(data.as_bytes())?;
        let _status = cmd.wait()?;

        // Write version downloaded to file
        File::create(format!("{}/nixpkgs.ver", sourcedir))?.write_all(latestpkgsver.as_bytes())?;
    } else {
        return Err(anyhow!("Failed to download latest packages.json"));
    }
    Ok(())
}
