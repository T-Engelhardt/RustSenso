mod senso;

fn main() {
    println!("{}", senso::urls::AUTHENTICATE);
    println!("{}", senso::urls::NEW_TOKEN);
    println!("{}", senso::urls::LOGOUT);
}
