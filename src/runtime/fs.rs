use std::path::Path;

use super::util::unblock;

pub async fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
    let path = path.as_ref().to_owned();
    unblock(move || std::fs::read(path)).await
}

pub async fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
    let path = path.as_ref().to_owned();
    let contents = contents.as_ref().to_owned();
    unblock(move || std::fs::write(path, contents)).await
}
