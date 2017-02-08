use capsule::Context;

pub fn build_command(_context: &Context, mut _args: Vec<String>)
    -> Result<i32, String>
{
    _args.insert(0, String::from("vagga _capsule build"));
    unimplemented!();
}
