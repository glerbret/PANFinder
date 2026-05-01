use std::cmp;

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
