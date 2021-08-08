use tealr::{
    rlu::{
        rlua::{FromLua, ToLua},
        TealData,
    },
    Direction, RluaUserData, TypeName, TypeWalker,
};

#[derive(Clone, Copy)]
pub struct TestFromAndBack {
    value: i64,
}

impl<'lua> FromLua<'lua> for TestFromAndBack {
    fn from_lua(lua_value: rlua::Value<'lua>, _: rlua::Context<'lua>) -> rlua::Result<Self> {
        match lua_value {
            rlua::Value::Integer(x) => Ok(TestFromAndBack { value: x }),
            _ => Err(rlua::Error::FromLuaConversionError {
                from: "unknown",
                to: "TestFromAndBack",
                message: Some("expected integer".to_string()),
            }),
        }
    }
}

impl<'lua> ToLua<'lua> for TestFromAndBack {
    #[allow(clippy::wrong_self_convention)]
    fn to_lua(self, lua: rlua::Context<'lua>) -> rlua::Result<rlua::Value<'lua>> {
        self.value.to_string().to_lua(lua)
    }
}
impl TypeName for TestFromAndBack {
    fn get_type_name(dir: Direction) -> std::borrow::Cow<'static, str> {
        match dir {
            Direction::FromLua => i64::get_type_name(Direction::FromLua),
            Direction::ToLua => String::get_type_name(Direction::ToLua),
        }
    }
}

#[derive(RluaUserData, TypeName)]
struct Holder {
    value: TestFromAndBack,
}
impl TealData for Holder {
    fn add_methods<'lua, T: tealr::rlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_function("TestBackAndForth", |_, value: TestFromAndBack| Ok(value));
        methods.add_method("get_value", |_, me, _: ()| Ok(me.value))
    }
}

#[test]
fn generate_correct_type() {
    let file_contents = TypeWalker::new()
        .process_type::<Holder>(tealr::Direction::ToLua)
        .generate_global("Example")
        .expect("oh no :(");

    assert_eq!(file_contents, "global record Example\n\trecord Holder\n\t\tuserdata\n\t\t-- Pure methods\n\t\tget_value: function(Holder):(string)\n\t\t-- Pure functions\n\t\tTestBackAndForth: function(integer):(string)\n\n\tend\nend\nreturn Example")
}
