use fit_rust::protocol::message_type::MessageType;
use fit_rust::protocol::FitMessage;
use fit_rust::Fit;
use std::fs;

fn main() {
    let file = fs::read("xingzhe/data/2023-05-10 上午 骑行.fit").unwrap();
    let fit: Fit = Fit::read(file).unwrap();
    for data in &fit.data {
        match data {
            FitMessage::Definition(_) => {}
            FitMessage::Data(msg) => match msg.data.message_type {
                MessageType::Session => {
                    println!("Data: {:?}", msg.data);
                }
                _ => {}
            },
        }
    }
}
