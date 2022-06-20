use std::collections::{BTreeMap, HashSet};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(feature="containers")]
use dkregistry::v2::Client as RegistryClient;

#[cfg(feature="containers")]
use futures::stream::StreamExt;

#[cfg(feature="containers")]
use indicatif::{ProgressBar, ProgressStyle};

#[cfg(feature="containers")]
use tar::{Entry, EntryType};

#[cfg(feature="containers")]
use tokio::{
    io::AsyncWriteExt,
    sync::oneshot,
};


#[cfg(feature="containers")]
use quire::{
    validate as V,
    ast::{Ast, ScalarKind, Tag},
};

#[cfg(feature="containers")]
use crate::{
    builder::commands::tarcmd::TarCmd,
    capsule::packages as capsule,
    container::util::clean_dir,
    file_util::{Dir, Lock, safe_remove},
};
use crate::build_step::{BuildStep, Config, Digest, Guard, StepError, VersionError};

pub const DEFAULT_REGISTRY_HOST: &str = "index.docker.io";
const DEFAULT_IMAGE_NAMESPACE: &str = "library";
const DEFAULT_IMAGE_TAG: &str = "latest";

const DOCKER_LAYERS_CACHE_PATH: &str = "/vagga/cache/docker-layers";

#[derive(Serialize, Deserialize, Debug)]
pub struct DockerImage {
    pub registry: String,
    pub image: String,
    pub tag: String,
    pub insecure: Option<bool>,
    pub path: PathBuf,
}

impl DockerImage {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("registry", V::Scalar::new().default(DEFAULT_REGISTRY_HOST))
        .member("image", V::Scalar::new())
        .member("tag", V::Scalar::new().default(DEFAULT_IMAGE_TAG))
        .member("insecure", V::Scalar::new().optional())
        .member("path", V::Directory::new().absolute(true).default("/"))
        .parser(parse_image)
    }
}

fn parse_image(ast: Ast) -> BTreeMap<String, Ast> {
    match ast {
        Ast::Scalar(pos, _, _, value) => {
            let mut map = BTreeMap::new();

            let (image, registry) = if let Some((registry, image)) = value.split_once('/') {
                if registry == "localhost" || registry.contains(|c| c == '.' || c == ':') {
                    map.insert(
                        "registry".to_string(),
                        Ast::Scalar(pos.clone(), Tag::NonSpecific, ScalarKind::Plain, registry.to_string())
                    );
                    (image, Some(registry))
                } else {
                    (value.as_str(), None)
                }
            } else {
                (value.as_str(), None)
            };

            let image = if let Some((image, tag)) = image.rsplit_once(':') {
                map.insert(
                    "tag".to_string(),
                    Ast::Scalar(pos.clone(), Tag::NonSpecific, ScalarKind::Plain, tag.to_string())
                );
                image
            } else {
                image
            };

            let image = if !image.contains('/') && registry.is_none() {
                format!("{}/{}", DEFAULT_IMAGE_NAMESPACE, image)
            } else {
                image.to_string()
            };

            map.insert(
                "image".to_string(),
                Ast::Scalar(pos.clone(), Tag::NonSpecific, ScalarKind::Plain, image)
            );

            map
        },
        _ => unreachable!(),
    }
}

impl BuildStep for DockerImage {
    fn name(&self) -> &'static str {
        "DockerImage"
    }

    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest) -> Result<(), VersionError> {
        hash.field("registry", &self.registry);
        hash.field("image", &self.image);
        hash.field("tag", &self.tag);
        hash.opt_field("insecure", &self.insecure);
        hash.field("path", &self.path);
        Ok(())
    }

    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, _build: bool) -> Result<(), StepError> {
        let registry = if let Some(registry) = guard.ctx.settings.docker_registry_aliases.get(&self.registry) {
            registry
        } else {
            &self.registry
        };
        let _image;
        let image = if registry == DEFAULT_REGISTRY_HOST && !self.image.contains("/") {
            _image = format!("library/{}", &self.image);
            &_image
        } else {
            &self.image
        };
        let insecure = self.insecure.unwrap_or_else(||
            is_insecure_registry(registry, &guard.ctx.settings.docker_insecure_registries)
        );
        if !insecure {
            capsule::ensure(&mut guard.ctx.capsule, &[capsule::Https])?;
        }
        Dir::new(DOCKER_LAYERS_CACHE_PATH)
            .recursive(true)
            .create()
            .map_err(|e|
                format!("Cannot create docker layers cache directory: {}", e)
            )?;
        let dst_path = Path::new("/vagga/root").join(&self.path.strip_prefix("/").unwrap());
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Error creating tokio runtime: {}", e))?
            .block_on(download_and_unpack_image(
                registry, insecure, image, &self.tag, &dst_path
            ))?;
        Ok(())
    }

    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

fn is_insecure_registry(
    registry: &str, insecure_registries: &HashSet<String>
) -> bool {
    let registry_host = match registry.split_once(':') {
        Some((host, _port)) => host,
        None => registry,
    };
    insecure_registries.contains(registry_host)
}

/// See:
/// - https://github.com/moby/moby/blob/v20.10.11/pkg/archive/whiteouts.go
/// - https://github.com/moby/moby/blob/v20.10.11/pkg/archive/diff.go#L131
#[cfg(feature="containers")]
fn whiteout_entry_handler(entry: &Entry<Box<dyn Read>>, dst_path: &Path) -> Result<bool, String> {
    let file_name = dst_path.file_name()
        .and_then(|fname| fname.to_str());
    let file_name = if let Some(file_name) = file_name {
        file_name
    } else {
        return Ok(false);
    };

    if entry.header().entry_type() != EntryType::Regular {
        return Ok(false);
    }

    if let Some(whiteout) = file_name.strip_prefix(".wh.") {
        let dir = dst_path.parent().unwrap();
        if whiteout == ".wh..opq" {
            // TODO: Track and keep files that were unpacked from the current archive
            clean_dir(dir, false)?
        } else {
            let mut whiteout_path = dir.to_path_buf();
            whiteout_path.push(whiteout);
            safe_remove(&whiteout_path)
                .map_err(|e| format!("Cannot remove {:?} path: {}", &whiteout_path, e))?;
        }
        return Ok(true);
    }

    Ok(false)
}

#[cfg(feature="containers")]
async fn download_and_unpack_image(
    registry: &str, insecure: bool, image: &str, tag: &str, dst_path: &Path
) -> Result<(), StepError> {
    let auth_scope = format!("repository:{}:pull", image);
    let client = build_client(registry, insecure, &[&auth_scope]).await?;

    println!("Downloading docker image: {}/{}:{}", registry, image, tag);
    let manifest = client.get_manifest(&image, &tag).await?;

    let layers_digests = manifest.layers_digests(None)?;

    let mut downloaded_layer_txs = vec!();
    let mut downloaded_layer_rxs = vec!();
    for _ in &layers_digests {
        let (tx, rx) = oneshot::channel();
        downloaded_layer_rxs.push(rx);
        downloaded_layer_txs.push(tx);
    }

    let dst_path = dst_path.to_path_buf();
    let unpack_task = tokio::spawn(async move {
        for layer_ch in downloaded_layer_rxs {
            match layer_ch.await {
                Ok((digest, layer_path)) => {
                    let dst_path = dst_path.clone();
                    if let Err(e) = unpack_layer(digest, layer_path, dst_path).await {
                        return Err(e);
                    }
                }
                Err(_) => {
                    // Channel is dropped if download task is cancelled
                },
            }
        }
        Ok(())
    });

    let image = image.to_string();
    let download_task = tokio::spawn(async move {
        for (digest, tx) in layers_digests.iter().zip(downloaded_layer_txs.into_iter()) {
            let digest = digest.clone();
            let client = client.clone();
            match download_blob(&client, &image, &digest).await {
                Ok(layer_path) => {
                    // Unpack task may be cancelled so ignore sending errors
                    tx.send((digest.clone(), layer_path)).ok();
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    });

    match download_task.await {
        Ok(Ok(_)) => {},
        Ok(Err(client_err)) => {
            unpack_task.abort();
            return Err(client_err.into());
        },
        Err(join_err) => {
            unpack_task.abort();
            return Err(
                format!("Error waiting a download layers task: {}", join_err).into()
            );
        },
    }

    match unpack_task.await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(unpack_err)) => Err(unpack_err.into()),
        Err(join_err) if join_err.is_cancelled() => Ok(()),
        Err(join_err) => Err(
            format!("Error waiting an unpack layers task: {}", join_err).into()
        ),
    }
}

async fn unpack_layer(
    digest: String, layer_path: PathBuf, dst_path: PathBuf
) -> Result<(), String> {
    let unpack_future_res = tokio::task::spawn_blocking(move || {
        println!("Unpacking docker layer: {}", digest);
        TarCmd::new(&layer_path, &dst_path)
            .preserve_owner(true)
            .override_entries(true)
            .entry_handler(whiteout_entry_handler)
            .unpack()
    }).await;
    unpack_future_res
        .map_err(|e| format!("Error waiting an unpack layer future: {}", e))?
        .map_err(|e| format!("Error unpacking docker layer: {}", e))
}

#[cfg(feature="containers")]
async fn build_client(
    registry: &str, insecure: bool, auth_scopes: &[&str]
) -> Result<Arc<RegistryClient>, StepError> {
    let client_config = RegistryClient::configure()
        .registry(registry)
        .insecure_registry(insecure)
        .username(None)
        .password(None);
    let client = client_config.build()?;

    let client = match client.is_auth().await {
        Ok(true) => client,
        Ok(false) => client.authenticate(auth_scopes).await?,
        Err(e) => return Err(e.into()),
    };
    Ok(Arc::new(client))
}

#[cfg(feature="containers")]
async fn download_blob(
    client: &RegistryClient, image: &str, layer_digest: &str
) -> Result<PathBuf, String> {
    let digest = layer_digest.split_once(':')
        .ok_or(format!("Invalid layer digest: {}", layer_digest))?
        .1;
    let short_digest = &digest[..12];

    let layers_cache = Path::new(DOCKER_LAYERS_CACHE_PATH);
    let blob_file_name = format!("{}.tar.gz", digest);
    let blob_path = layers_cache.join(&blob_file_name);
    match tokio::fs::symlink_metadata(&blob_path).await {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let lock_file_name = format!(".{}.lock", &blob_file_name);
            let lock_msg = format!("Another process downloads blob: {}", &short_digest);
            let lock_fut = tokio::task::spawn_blocking(move || {
                let lockfile = layers_cache.join(lock_file_name);
                Lock::exclusive_wait(lockfile, true, &lock_msg)
            });
            let _lock = lock_fut.await
                .map_err(|e| format!("Error waiting a lock file future: {}", e))?
                .map_err(|e| format!("Error taking exclusive lock: {}", e))?;

            match tokio::fs::symlink_metadata(&blob_path).await {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::NotFound => {
                    let blob_tmp_file_name = format!(".{}.tmp", &blob_file_name);
                    let blob_tmp_path = layers_cache.join(&blob_tmp_file_name);

                    println!("Downloading docker layer: {}", &layer_digest);
                    let blob_resp = client.get_blob_response(image, layer_digest).await
                        .map_err(|e| format!("Error getting docker blob response: {}", e))?;
                    let blob_size = blob_resp.size();
                    let mut blob_stream = blob_resp.stream();
                    let mut blob_file = tokio::fs::File::create(&blob_tmp_path).await
                        .map_err(|e| format!("Cannot create layer file: {}", e))?;

                    let progress = if let Some(blob_size) = blob_size {
                        ProgressBar::new(blob_size)
                            .with_style(
                                ProgressStyle::default_bar()
                                    .template("{msg}: {percent}%[{bar:40}] {bytes}/{total_bytes} {bytes_per_sec} {eta}")
                                    .progress_chars("=> ")
                            )
                    } else {
                        ProgressBar::new_spinner()
                            .with_style(
                                ProgressStyle::default_bar()
                                    .template("{msg}: [{spinner:40}] {bytes} {bytes_per_sec}")
                                    .progress_chars("=> ")
                            )
                    };
                    progress.set_message(short_digest.to_string());
                    progress.set_draw_rate(5);

                    while let Some(chunk) = blob_stream.next().await {
                        let chunk = chunk.map_err(|e| format!("Error fetching layer chunk: {}", e))?;
                        blob_file.write_all(&chunk).await
                            .map_err(|e| format!("Cannot write blob file: {}", e))?;
                        progress.inc(chunk.len() as u64);
                    }
                    progress.finish_and_clear();

                    tokio::fs::rename(&blob_tmp_path, &blob_path).await
                        .map_err(|e| format!("Cannot rename docker blob: {}", e))?;
                }
                Err(e) => return Err(format!("{}", e)),
            }

        }
        Err(e) => return Err(format!("{}", e)),
    }
    Ok(blob_path)
}
