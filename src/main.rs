#![allow(unused)]
use futures::join;
use std::rc::Rc;
use std::thread;
use std::time;
use tokio::time as tokio_time;

async fn async_function(message: &str) -> &str {
    tokio_time::sleep(time::Duration::from_millis(2000)).await;
    println!("{}", message);
    message
}

pub struct Baz {
    v: String,
}

pub struct Whatever {
    a: Vec<Baz>,
}

struct Coercion<T> {
    value: Option<T>,
}

#[tokio::main]
async fn main() {
    let handler = thread::spawn(|| println!("HELLO THREAD"));
    // handler.join().unwrap();
    let results = join!(
        async_function("TEST!"),
        async_function("TEST2!"),
        async_function("TEST3!")
    );

    let mut coercion: Coercion<i64> = Coercion { value: None };

    let p: f64 = 1.0;

    coercion.value = Some(p as i64);

    let what = Rc::new(Whatever {
        a: vec![Baz {
            v: "test".to_string(),
        }],
    });

    println!("Results: {:?}", results);

    let results = join!(
        async_function("TEST4!"),
        async_function("TEST5!"),
        async_function("TEST6!")
    );

    println!("Results: {:?}", results);

    println!("Hello, world!");
}
