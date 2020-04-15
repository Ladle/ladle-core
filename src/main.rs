use ladl_core::text::Input;

fn main() {
    let i = Input::new("1\n12\n123\n1234\n12345\n123456".into());

    println!("{}", i.get_span(2, 10, "AAAAAAA"));
}
