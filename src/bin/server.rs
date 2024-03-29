use phoenix_rec::data_types::DataType;
use phoenix_rec::save_data;
use phoenix_rec::server::{create_server, stop_server};

fn main() {
    create_server();
    save_data(vec![DataType::Distance(0)]);
    save_data(vec![DataType::CalcSpeed(0, 0)]);
    stop_server();
    loop {}
}