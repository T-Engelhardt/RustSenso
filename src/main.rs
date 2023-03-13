use crate::senso::connector::Connector;

mod senso;

fn main() {
    /*
    println!("{}", senso::urls::AUTHENTICATE);
    println!("{}", senso::urls::NEW_TOKEN);
    println!("{}", senso::urls::LOGOUT);
    let serial = "1234";
    println!("{}", senso::urls::LIVE_REPORT(serial));
    println!("{}", senso::urls::SYSTEM_STATUS(serial));
    println!(
        "{}",
        senso::urls::EMF_REPORT_DEVICE(
            serial,
            "NoneGateway-LL_HMU03_0351_HP_Platform_Outdoor_Monobloc_PR_EBUS"
        )
    );
    */
    let mut c = Connector::new("21223900202609620938071939N6".into()).unwrap();
    c.login("T.Engelhardt", "vZW5Sz4Xmj#I").unwrap();
    // TODO macro?? try x time before giving up
    c.system_status().unwrap();
}
