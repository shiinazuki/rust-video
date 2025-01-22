# rust-video

可以使用sqlx-cli 来创建表
安装sqlx-cli

cargo install sqlx-cli --no-default-features --features rustls --features postgres
 
 sqlx migrate add initial
 
 可以将相关建表语句写在initial.sql中
 
 touch .env文件
 
 在.env文件中写入数据库链接信息
 DATABASE_URL="postgres://"
 
 sqlx migrate run