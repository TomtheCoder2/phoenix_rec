use phoenix_rec::data_types::DataType;
use phoenix_rec::server::{add_data, create_server};
use phoenix_rec::Data;

fn main() {
    create_server();
    add_data(Data::RecordData(10, vec![DataType::Distance(10)]));
    add_data(Data::RecordData(11, vec![DataType::Distance(11)]));
    loop {}
}