use gol_engines::*;

fn main() {
    let otca_depth = 2;
    let top_pattern = vec![
        vec![0, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 1, 0, 0, 0, 0, 0],
        vec![1, 1, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let timer = std::time::Instant::now();

    let mut engine = HashLifeEngineAsync::from_recursive_otca_metapixel(otca_depth, top_pattern);
    println!("Time on building field: {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    let steps_log2 = 23;
    engine.update(steps_log2, Topology::Unbounded);
    println!("Time on big update: {:?}", timer.elapsed());
    assert_eq!(engine.hash(), 0xf35ef0ba0c9db279);
    assert_eq!(engine.population(), 6_094_494_746_384.0);
}

// use paste::paste;

// // The macro generates specific Fibonacci functions based on three parameters: i, j, and k.
// macro_rules! generate_fibonacci_function {
//     // Generate a function fibonacci_i using functions fibonacci_j and fibonacci_k
//     ($i:expr, $j:expr, $k:expr) => {
//         paste! {
//             pub async fn [<fibonacci_ $i>]() -> u64 {
//                 let fib_j = tokio::spawn([<fibonacci_ $j>]());
//                 let fib_k = tokio::spawn([<fibonacci_ $k>]());
//                 fib_j.await.unwrap() + fib_k.await.unwrap()
//                 // let fib_j = [<fibonacci_ $j>]();
//                 // let fib_k = [<fibonacci_ $k>]();
//                 // fib_j.await + fib_k.await
//             }
//         }
//     };
// }

// pub async fn fibonacci_0() -> u64 {
//     0
// }

// pub async fn fibonacci_1() -> u64 {
//     1
// }

// generate_fibonacci_function!(2, 0, 1);
// generate_fibonacci_function!(3, 1, 2);
// generate_fibonacci_function!(4, 2, 3);
// generate_fibonacci_function!(5, 3, 4);
// generate_fibonacci_function!(6, 4, 5);
// generate_fibonacci_function!(7, 5, 6);
// generate_fibonacci_function!(8, 6, 7);
// generate_fibonacci_function!(9, 7, 8);
// generate_fibonacci_function!(10, 8, 9);
// generate_fibonacci_function!(11, 9, 10);
// generate_fibonacci_function!(12, 10, 11);
// generate_fibonacci_function!(13, 11, 12);
// generate_fibonacci_function!(14, 12, 13);
// generate_fibonacci_function!(15, 13, 14);
// generate_fibonacci_function!(16, 14, 15);
// generate_fibonacci_function!(17, 15, 16);
// generate_fibonacci_function!(18, 16, 17);
// generate_fibonacci_function!(19, 17, 18);
// generate_fibonacci_function!(20, 18, 19);
// generate_fibonacci_function!(21, 19, 20);
// generate_fibonacci_function!(22, 20, 21);
// generate_fibonacci_function!(23, 21, 22);
// generate_fibonacci_function!(24, 22, 23);
// generate_fibonacci_function!(25, 23, 24);
// generate_fibonacci_function!(26, 24, 25);
// generate_fibonacci_function!(27, 25, 26);
// generate_fibonacci_function!(28, 26, 27);
// generate_fibonacci_function!(29, 27, 28);
// generate_fibonacci_function!(30, 28, 29);
// generate_fibonacci_function!(31, 29, 30);
// generate_fibonacci_function!(32, 30, 31);
// generate_fibonacci_function!(33, 31, 32);
// generate_fibonacci_function!(34, 32, 33);
// generate_fibonacci_function!(35, 33, 34);
// generate_fibonacci_function!(36, 34, 35);

// // async fn fib_async(x: u64) -> u64 {
// //     match x {
// //         0 => 0,
// //         1 => 1,
// //         _ => Box::pin(fib_async(x - 1)).await + Box::pin(fib_async(x - 2)).await,
// //     }
// // }

// // fn fib(x: u64) -> u64 {
// //     match x {
// //         0 => 0,
// //         1 => 1,
// //         _ => fib(x - 1) + fib(x - 2),
// //     }
// // }

// fn main() {
//     use std::time::Instant;
//     let timer = Instant::now();

//     tokio::runtime::Builder::new_current_thread()
//         .build()
//         .unwrap()
//         .block_on(async {
//             let result = fibonacci_36().await;
//             // let result = fib_async(36).await;
//             println!("Time on fib: {:?}", timer.elapsed());
//             assert_eq!(result, 14930352); // 6765);
//         });
//     // assert_eq!(fib(36), 14930352);
// }
