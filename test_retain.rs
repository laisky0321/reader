fn main() {
    let mut events = vec![1, 2, 3];
    events.retain(|event| {
        let is_two = *event == 2;
        if is_two {
            println!("Found two");
            return false;
        }
        true
    });
    println!("{:?}", events);
}
