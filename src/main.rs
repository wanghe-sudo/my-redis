use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use mini_redis::{Connection, Frame};
use mini_redis::Command::Set;
use tokio::net::{TcpListener, TcpStream};

// 创建一个新的数据类型
type Db = Arc<Mutex<HashMap<String, Bytes>>>;
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9379").await.unwrap();
    println!("Listening");
    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        println!("Accepted");
        // 每次有新的连接，则创建一个绿色线程
        // 并将socket和db变量的所有权移动到闭包中 
        tokio::spawn(async move {
            process(socket, db).await
        });
    }
}
/// 这里Db类型是之前是与哦给你type定义好的
async fn process(socket: TcpStream, db: Db) {
    // 在方法中引入对应的依赖
    use mini_redis::Command::{self, Get, Set};


    // 创建socket连接
    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                // 值被存储为 `Vec<u8>` 的形式
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                // 获取到db
                let mut db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd)
        };
        // 将请求响应返回给客户端
        connection.write_frame(&response).await.unwrap();
    };
}