use relm4::factory::FactoryComponent;

#[derive(Debug)]
struct TestFactoryComponent;

#[relm4_macros::factory]
impl FactoryComponent for TestFactoryComponent {

}

fn main() {}
