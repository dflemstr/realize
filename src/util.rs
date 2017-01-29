use std::io;

use sha1;

use error;

pub fn sha1<R>(mut read: R) -> error::Result<sha1::Digest>
    where R: io::Read
{
    let mut sha1 = sha1::Sha1::new();

    // SHA1 acts on 64-byte blocks, so this makes sense as a buffer size
    let mut buf = [0; 64];

    loop {
        let n = read.read(&mut buf)?;

        if n == 0 {
            break;
        }

        sha1.update(&buf[..n]);
    }

    Ok(sha1.digest())
}
