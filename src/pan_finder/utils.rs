use std::{cmp, fs, path::Path};

/// Read the first n bytes of a file
pub fn read_up_to(
    file: &mut impl std::io::Read,
    mut buf: &mut [u8],
) -> Result<usize, std::io::Error> {
    let buf_len = buf.len();

    while !buf.is_empty() {
        match file.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(buf_len - buf.len())
}

/// Check if a file is a text one
///
/// _Note:_ a file is considered be a text one if there is no `0` in its first bytes
pub fn is_text_file(data: &[u8], len: usize) -> bool {
    !data[0..cmp::min(len, 2000)].contains(&0u8)
}

/// Check if a file is a tar archive
pub fn is_tar_file(data: &[u8], len: usize) -> bool {
    if len >= 262 {
        let magic_number: [u8; 5] = data[257..262].try_into().unwrap();
        &magic_number == b"ustar"
    } else {
        false
    }
}

/// Check if a file is a text one
pub fn is_pdf_file(data: &[u8], len: usize) -> bool {
    if len >= 4 {
        let header: [u8; 4] = data[0..4].try_into().unwrap();
        &header == b"%PDF"
    } else {
        false
    }
}

/// Check if a file is a gzip one
pub fn is_gz_file(data: &[u8], len: usize) -> bool {
    if len >= 3 {
        let header: [u8; 3] = data[0..3].try_into().unwrap();
        &header == b"\x1F\x8B\x08"
    } else {
        false
    }
}

/// Check if a file is empty
pub fn is_file_empty(path: &Path) -> bool {
    match fs::metadata(path).map(|metadata| metadata.len() == 0) {
        Ok(res) => res,
        Err(err) => {
            println!(
                "Emptiness of {} cannot be checked ({}), file ignored",
                path.display(),
                err
            );
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_empty() {
        assert!(!is_file_empty(Path::new("./testdata/lister/text_file.txt")));
        assert!(is_file_empty(Path::new("./testdata/lister/empty_file")));
        assert!(is_file_empty(Path::new("./testdata/lister/not_exist")));
    }

    #[test]
    fn test_is_text_file() {
        let mut data: [u8; 2000] = [0x30; 2000];
        data[150] = 0;

        assert!(is_text_file(&data, 100));
        assert!(is_text_file(&data, 0));
        assert!(!is_text_file(&data, 200));
    }

    #[test]
    fn test_is_pdf_file() {
        let mut data: [u8; 2000] = [0x30; 2000];
        assert!(!is_pdf_file(&data, 100));

        data[0] = b'%';
        data[1] = b'P';
        data[2] = b'D';
        data[3] = b'F';
        assert!(is_pdf_file(&data, 100));
        assert!(!is_pdf_file(&data, 3));
    }

    #[test]
    fn test_is_gz_file() {
        let mut data: [u8; 2000] = [0x30; 2000];
        assert!(!is_gz_file(&data, 100));

        data[0] = b'\x1F';
        data[1] = b'\x8B';
        data[2] = b'\x08';
        assert!(is_gz_file(&data, 100));
        assert!(!is_gz_file(&data, 2));
    }

    #[test]
    fn test_is_tar_file() {
        let mut data: [u8; 2000] = [0x30; 2000];
        assert!(!is_tar_file(&data, 500));

        data[257] = b'u';
        data[258] = b's';
        data[259] = b't';
        data[260] = b'a';
        data[261] = b'r';
        assert!(is_tar_file(&data, 500));
        assert!(!is_tar_file(&data, 140));
    }
}
