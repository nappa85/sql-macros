use sql_macros::select;

#[test]
fn it_works() {
    select!("SELECT id FROM users");
}
