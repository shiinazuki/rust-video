use anyhow::Result;
use bytes::{BufMut, BytesMut};

fn main() -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    buf.extend_from_slice(b"hello world\n");
    buf.put(&b"abc`"[..]);
    buf.put_i64(0xdeadbeff);

    println!("buf={:?}", buf);
    let a = buf.split();
    println!("a={:?}", a);
    let mut b = a.freeze();
    println!("b={:?}", b);
    let c = b.split_to(12);
    println!("c={:?}", c);
    println!("b={:?}", b);
    println!("buf={:?}", buf);

    Ok(())
}
