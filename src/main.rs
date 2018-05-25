struct STestStruct {
    x : u64,
    blech : String,
    y : i32,
}

fn main() {
    let x : u64 = 64;
    let teststruct = STestStruct {
        x : 24,
        blech : "poopsock".to_string(),
        y : -5,
    };
    println!("Hello, world {}!", x);
    println!("Teststruct: {}, {}, {}", teststruct.x, teststruct.blech, teststruct.y);
}
