use log::warn;
use senso::connector::Connector;

fn main() {
    env_logger::init();

    let c = Connector::new("21223900202609620938071939N6".into());
    c.login("T.Engelhardt", "vZW5Sz4Xmj#I").unwrap();

    // TODO macro?? try x time before giving up
    let status = c.system_status().unwrap();
    println!("{:#?}", status);

    let live_report = c.live_report().unwrap();
    println!("{:#?}", live_report);

}