use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Clone)]
pub enum FSNode {
    File {
        content: Vec<u8>,
        created: u64,
        modified: u64,
    },
    Directory {
        entries: BTreeMap<String, FSNode>,
        created: u64,
        modified: u64,
    },
}

lazy_static! {
    static ref FS_ROOT: Mutex<FSNode> = Mutex::new(FSNode::Directory {
        entries: BTreeMap::new(),
        created: 0,
        modified: 0,
    });
}

lazy_static! {
    static ref CURRENT_PATH: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn list_current_directory() -> Vec<(String, bool)> {
    let current_path = CURRENT_PATH.lock();
    let fs = FS_ROOT.lock();
    let mut current = &*fs;

    for dir in current_path.iter() {
        if let FSNode::Directory { ref entries, .. } = current {
            if let Some(next) = entries.get(dir) {
                current = next;
            } else {
                return Vec::new();
            }
        }
    }

    let mut result = Vec::new();
    if let FSNode::Directory {
        entries: ref dir_entries,
        ..
    } = current
    {
        for (name, node) in dir_entries.iter() {
            result.push((name.clone(), matches!(node, FSNode::Directory { .. })));
        }
    }

    result.sort();
    result
}

pub fn list_directory() -> Vec<(String, bool)> {
    let fs = FS_ROOT.lock();
    let mut result = Vec::new();

    if let FSNode::Directory {
        entries: ref dir_entries,
        ..
    } = *fs
    {
        for (name, node) in dir_entries.iter() {
            result.push((name.clone(), matches!(node, FSNode::Directory { .. })));
        }
    }

    result.sort();
    result
}

pub fn create_directory(path: &str) -> Result<(), &'static str> {
    let mut fs = FS_ROOT.lock();
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    fn create_dir_recursive(node: &mut FSNode, parts: &[&str]) -> Result<(), &'static str> {
        if parts.is_empty() {
            return Ok(());
        }

        match node {
            FSNode::Directory { entries, .. } => {
                let part = parts[0];
                if !entries.contains_key(part) {
                    entries.insert(
                        String::from(part),
                        FSNode::Directory {
                            entries: BTreeMap::new(),
                            created: 0,
                            modified: 0,
                        },
                    );
                }

                if let Some(next) = entries.get_mut(part) {
                    create_dir_recursive(next, &parts[1..])
                } else {
                    Err("Failed to create directory")
                }
            }
            _ => Err("Not a directory"),
        }
    }

    create_dir_recursive(&mut fs, &parts)
}

pub fn read_file(path: &str) -> Result<Vec<u8>, &'static str> {
    let fs = FS_ROOT.lock();

    if let FSNode::Directory { ref entries, .. } = *fs {
        if let Some(FSNode::File { ref content, .. }) = entries.get(path) {
            Ok(content.clone())
        } else {
            Err("File not found")
        }
    } else {
        Err("Root is not a directory")
    }
}

pub fn create_file(path: &str, content: Option<Vec<u8>>) -> Result<(), &'static str> {
    let mut fs = FS_ROOT.lock();
    let current_path = CURRENT_PATH.lock();

    let mut current = &mut *fs;
    for dir in current_path.iter() {
        if let FSNode::Directory {
            ref mut entries, ..
        } = current
        {
            current = entries.get_mut(dir).ok_or("Current directory not found")?;
        } else {
            return Err("Current path is not a directory");
        }
    }

    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let filename = parts.last().ok_or("Invalid path")?;
    let parent_dirs = &parts[..parts.len() - 1];

    for &dir in parent_dirs {
        if let FSNode::Directory {
            ref mut entries, ..
        } = current
        {
            current = entries
                .entry(String::from(dir))
                .or_insert_with(|| FSNode::Directory {
                    entries: BTreeMap::new(),
                    created: 0,
                    modified: 0,
                });
        } else {
            return Err("Path component is not a directory");
        }
    }

    if let FSNode::Directory {
        ref mut entries, ..
    } = current
    {
        entries.insert(
            String::from(*filename),
            FSNode::File {
                content: content.unwrap_or_default(),
                created: 0,
                modified: 0,
            },
        );
        Ok(())
    } else {
        Err("Parent is not a directory")
    }
}

pub fn write_file(path: &str, content: &[u8], append: bool) -> Result<(), &'static str> {
    let mut fs = FS_ROOT.lock();

    if let FSNode::Directory {
        ref mut entries, ..
    } = *fs
    {
        if append {
            if let Some(FSNode::File {
                content: ref mut file_content,
                ..
            }) = entries.get_mut(path)
            {
                file_content.extend_from_slice(content);
            } else {
                entries.insert(
                    String::from(path),
                    FSNode::File {
                        content: content.to_vec(),
                        created: 0,
                        modified: 0,
                    },
                );
            }
        } else {
            entries.insert(
                String::from(path),
                FSNode::File {
                    content: content.to_vec(),
                    created: 0,
                    modified: 0,
                },
            );
        }
        Ok(())
    } else {
        Err("Root is not a directory")
    }
}

pub fn change_directory(path: &str) -> Result<(), &'static str> {
    let mut current_path = CURRENT_PATH.lock();
    match path {
        "/" => {
            current_path.clear();
            Ok(())
        }
        ".." => {
            if !current_path.is_empty() {
                current_path.pop();
                Ok(())
            } else {
                Err("Already at root directory")
            }
        }
        path => {
            let fs = FS_ROOT.lock();
            let mut node = &*fs;

            for dir in current_path.iter() {
                if let FSNode::Directory { ref entries, .. } = node {
                    if let Some(next) = entries.get(dir) {
                        node = next;
                    } else {
                        return Err("Path not found");
                    }
                }
            }

            if let FSNode::Directory { ref entries, .. } = node {
                if let Some(FSNode::Directory { .. }) = entries.get(path) {
                    current_path.push(String::from(path));
                    Ok(())
                } else {
                    Err("Directory not found")
                }
            } else {
                Err("Not a directory")
            }
        }
    }
}

pub fn get_current_path() -> Vec<String> {
    CURRENT_PATH.lock().clone()
}
