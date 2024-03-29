use rec::data_types::DataType;
use tcp::server::{add_data, create_server};
use rec::Data;

fn main() {
    create_server();
    add_data(Data::RecordData(10, vec![DataType::Distance(10)]));
    add_data(Data::RecordData(11, vec![DataType::Distance(11)]));
    loop {}
}