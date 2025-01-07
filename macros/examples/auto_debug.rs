use macros::AutoDebug;

#[allow(unused)]
#[derive(AutoDebug)]
struct RespBulkString {
    inner: String,

    #[debug(skip)]
    nothing: (),

    hello: u32,
}

fn main() {
    let s = RespBulkString {
        inner: "hello".to_string(),
        nothing: (),
        hello: 44,
    };

    println!("{:?}", s);
}
