//! Spike 0: prove a Rust host can drive the embedded bytecode Toploop in-process.

ocaml::import! {
    fn eval_string(src: String) -> isize;
}

fn main() {
    let rt = ocaml::runtime::init();

    let two = unsafe { eval_string(&rt, "1 + 1".to_string()) }.expect("eval 1 + 1");
    println!("eval(\"1 + 1\") = {two}");
    assert_eq!(two, 2);

    let answer = unsafe { eval_string(&rt, "(print_string \"hi from ocaml\\n\"; 42)".to_string()) }
        .expect("eval print_string");
    println!("eval(print_string; 42) = {answer}");
    assert_eq!(answer, 42);

    println!("SPIKE 0 PASS");
}
