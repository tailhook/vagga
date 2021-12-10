use std::collections::{BTreeMap, HashSet};
use std::fs::{remove_dir_all, remove_file};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(feature="containers")]
use dkregistry::v2::Client as RegistryClient;

#[cfg(feature="containers")]
use futures::stream::StreamExt;

#[cfg(feature="containers")]
use tar::{Entry, EntryType};

#[cfg(feature="containers")]
use tokio::{
    io::AsyncWriteExt,
    sync::Semaphore,
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
    file_util::{Dir, Lock},
};
use crate::build_step::{BuildStep, Config, Digest, Guard, StepError, VersionError};

const DEFAULT_REGISTRY_HOST: &str = "registry-1.docker.io";
const DEFAULT_IMAGE_NAMESPACE: &str = "library";
const DEFAULT_IMAGE_TAG: &str = "latest";

const DOCKER_LAYERS_CACHE_PATH: &str = "/vagga/cache/docker-layers";
const DOCKER_LAYERS_DOWNLOAD_CONCURRENCY: usize = 4;

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
        let insecure = self.insecure.unwrap_or_else(|| {
            is_insecure_registry(&self.registry, &guard.ctx.settings.docker_insecure_registries)
        });
        if !insecure {
            capsule::ensure(&mut guard.ctx.capsule, &[capsule::Https])?;
        }
        Dir::new(DOCKER_LAYERS_CACHE_PATH)
            .recursive(true)
            .create()
            .expect("Docker layers cache dir");
        let layer_paths = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Tokio runtime")
            .block_on(download_image(&self.registry, insecure, &self.image, &self.tag))
            .expect("Downloaded layers");
        let dst_path = Path::new("/vagga/root").join(&self.path.strip_prefix("/").unwrap());
        for layer_path in layer_paths.iter() {
            TarCmd::new(layer_path, &dst_path)
                .preserve_owner(true)
                .entry_handler(whiteout_entry_handler)
                .unpack()?;
        }
        Ok(())
    }

    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
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
            if whiteout_path.is_dir() {
                remove_dir_all(&whiteout_path)
                    .map_err(|e| format!("Cannot remove directory: {}", e))?;
            } else {
                remove_file(whiteout_path)
                    .map_err(|e| format!("Cannot delete file: {}", e))?;
            }
        }
        return Ok(true);
    }

    Ok(false)
}

fn is_insecure_registry(registry: &str, insecure_registries: &HashSet<String>) -> bool {
    let registry_url = url::Url::parse(&format!("http://{}", registry)).unwrap();
    let registry_host = registry_url.domain().unwrap();
    insecure_registries.contains(registry_host)
}

#[cfg(feature="containers")]
async fn download_image(
    registry: &str, insecure: bool, image: &str, tag: &str
) -> Result<Vec<PathBuf>, StepError> {
    let auth_scope = format!("repository:{}:pull", image);
    let client = build_client(registry, insecure, &[&auth_scope]).await?;

    println!("Downloading docker image: {}/{}:{}", registry, image, tag);
    let manifest = client.get_manifest(&image, &tag).await?;

    let layers_digests = manifest.layers_digests(None)?;

    let layers_download_semaphore = Arc::new(
        Semaphore::new(DOCKER_LAYERS_DOWNLOAD_CONCURRENCY)
    );
    let layers_futures = layers_digests.iter()
        .map(|digest| {
            let image = image.to_string();
            let digest = digest.clone();
            let client = client.clone();
            let sem = layers_download_semaphore.clone();
            tokio::spawn(async move {
                if let Ok(_guard) = sem.acquire().await {
                    info!("Downloading docker layer: {}", &digest);
                    download_blob(&client, &image, &digest).await
                } else {
                    panic!("Semaphore was closed unexpectedly")
                }
            })
        })
        .collect::<Vec<_>>();
    let mut layers_paths = vec!();
    let mut layers_errors = vec!();
    for layer_res in futures::future::join_all(layers_futures).await.into_iter() {
        match layer_res {
            Ok(Ok(layer)) => layers_paths.push(layer),
            Ok(Err(client_err)) => layers_errors.push(client_err),
            Err(join_err) => layers_errors.push(format!("{}", join_err)),
        }
    }
    if !layers_errors.is_empty() {
        Err(layers_errors.into())
    } else {
        Ok(layers_paths)
    }
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
    let digest = layer_digest.split_once(':').unwrap().1;
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

                    println!("Downloading docker blob: {}", &short_digest);
                    let mut blob_stream = client.get_blob_stream(image, layer_digest).await
                        .expect("Get blob response");
                    let mut blob_file = tokio::fs::File::create(&blob_tmp_path).await
                        .expect("Create layer file");
                    while let Some(chunk) = blob_stream.next().await {
                        let chunk = chunk.expect("Layer chunk");
                        blob_file.write_all(&chunk).await
                            .map_err(|e| format!("Cannot write blob file: {}", e))?;
                    }
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