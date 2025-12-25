use std::{
    env,
    fs::{self, DirEntry},
    path::PathBuf,
    ops::{Deref, DerefMut},
    os::unix::fs::PermissionsExt,
    sync::{Arc, OnceLock, RwLock},
    collections::{BTreeMap, HashMap},
};

use ordered_float::OrderedFloat;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use strsim::jaro_winkler;

use crate::{
    descriptions::{
        Description,
        get_description,
        insert_description,
    },
    man::get_manpaths,
    roff::extract_description_section,
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Binary {
    pub name: String,
    pub manpath: Option<PathBuf>,
}

impl Binary {
    pub fn extract_description(&self) -> Option<Description> {
        let path = match self.manpath.as_ref() {
            Some(p) => p,
            None => return None,
        };

        let buf = path.as_path().try_into();

        buf.ok()
            .and_then(|buf| extract_description_section(buf))
            .and_then(|section| section.get_description())
    }

    pub fn get_description(&self) -> Option<Arc<Description>>{
        if let Some(description) = get_description(self) {
            return Some(description)
        }

        let description = self
            .extract_description()
            .map(|desc| Arc::new(desc));

        if let Some(desc) = &description {
            insert_description(self, desc.clone());
        }

        description
    }
}

pub type BinaryNode = Arc<RwLock<Binary>>;
pub type Binaries = HashMap<String, BinaryNode>;

static SEARCH_PATHS: OnceLock<Vec<PathBuf>> = OnceLock::new();
static BINARIES: OnceLock<Binaries> = OnceLock::new();

pub fn init_search_path(path: &String) {
    SEARCH_PATHS.get_or_init(|| {
        env::split_paths(&path)
            .collect::<Vec<PathBuf>>()
    });
}

fn init_binaries() -> Binaries {
    let mut binaries = Binaries::new();

    if SEARCH_PATHS.get().is_none()
    && let Some(v) = env::var_os("PATH")
    {
        let path = v
            .to_string_lossy()
            .to_string();

        init_search_path(&path);
    };

    let paths = SEARCH_PATHS.get().unwrap();

    let dirs = paths
        .into_iter()
        .map(|path| fs::read_dir(&path))
        .flatten();

    let entries = dirs
        .map(|dir| dir.into_iter())
        .flatten();

    for entry in entries {
        let entry = match entry {
            Ok(v) => v,
            Err(_) => continue,
        };

        if !is_entry_executable(&entry) {
            continue;
        }

        let name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        binaries.insert(
            name.clone(),
            Arc::new(RwLock::new(Binary {
                name: name.clone(),
                manpath: None,
            })),
        );
    }

    binaries
}

pub fn is_binary_exist(name: &str) -> bool {
    let binaries = BINARIES.get_or_init(init_binaries);
    binaries.contains_key(name)
}

fn is_entry_executable(entry: &DirEntry) -> bool {
    let metadata = match fs::metadata(entry.path()) {
        Ok(md) => md,
        Err(_) => return false,
    };
    
    if metadata.is_dir() {
        return false;
    }

    let mode = metadata.permissions().mode();
    let x_perm = 0o111;

    mode & x_perm != 0
}

type BinSearchResultInner =
    BTreeMap<OrderedFloat<f64>, BinaryNode>;

#[derive(Default, Clone)]
pub struct BinSearchResult {
    inner: BinSearchResultInner,
}

impl Deref for BinSearchResult {
    type Target = BinSearchResultInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for BinSearchResult {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl BinSearchResult {
    #[inline]
    pub fn ordered_iter(&self)
        -> impl Iterator<Item = &BinaryNode>
    {
        self.values().rev()
    }

    #[inline]
    pub fn owned_ordered_iter(&self)
        -> impl Iterator<Item = BinaryNode>
    {
        self.values().rev().map(|v| v.clone())
    }
}

pub fn search_binaries(search: &str) -> BinSearchResult {
    let binaries = BINARIES.get_or_init(init_binaries);

    let get_similarity = |val: &str| jaro_winkler(val, search);

    binaries
        .par_iter()
        .map(|(_, binary)| {
            let readable_binary = binary.read().unwrap();

            let similarity = get_similarity(&readable_binary.name.as_str());
            let key = OrderedFloat(similarity);

            (key, binary)
        })
        .fold_with(BinSearchResult::default(), |mut acc, (k, v)| {
            acc.insert(k, v.clone());
            acc
        })
        .reduce_with(|mut acc1, acc2| {
            for (k, v) in acc2.iter() {
                acc1.entry(*k).or_insert(v.clone());
            }
            acc1
        })
        .unwrap_or_else(BinSearchResult::default)
}

pub fn attach_manpaths(binaries: &Vec<BinaryNode>) {
    let attachables = binaries
        .iter()
        .filter(|binary| binary.read().unwrap().manpath.is_none());

    let names = attachables.clone()
        .map(|binary| binary.read().unwrap().name.clone())
        .collect::<Vec<String>>();

    if names.is_empty() {
        return;
    }

    let manpaths = match get_manpaths(names) {
        Some(v) => v,
        None => return,
    };

    let mut paths = manpaths.paths.iter().peekable();
    let not_founds = manpaths.not_founds;

    for attachable in attachables {
        let mut writeable_binary = attachable.write().unwrap();
        let name = &writeable_binary.name;

        if not_founds.contains(&name) {
            writeable_binary.manpath = None;
        } else {
            let manpath = paths.next().map(PathBuf::from);
            writeable_binary.manpath = manpath;
        }

        drop(writeable_binary);
    }
}
