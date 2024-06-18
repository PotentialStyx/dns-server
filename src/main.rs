mod parser;

fn main() {
    println!("{}", parser::OpCode::Query);
    println!("{}", parser::OpCode::IQuery);
    println!("{}", parser::OpCode::Status);
    println!("{}", parser::OpCode::Reserved(5));
}
