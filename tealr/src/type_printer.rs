
use crate::{teal_data::TealData, teal_data_methods::TealDataMethods};
use crate::teal_multivalue::TealMultiValue;

use rlua::{Context, FromLuaMulti, MetaMethod, Result, ToLuaMulti, UserData};

///just a temprorary object so I could easily test my current code.
pub struct TypePrinter;
impl<'lua, T> TealDataMethods<'lua, T> for TypePrinter where T: 'static + TealData + UserData {
    fn add_method<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R> {
        println!("method: {:?} gets: {:?}, returns {:?}", std::str::from_utf8(name.as_ref()) , A::get_types(),R::get_types());
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R> {
            println!("method_mut: {:?} gets: {:?}, returns {:?}", std::str::from_utf8(name.as_ref()) , A::get_types(),R::get_types());
    }

    fn add_function<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R> {
            println!("function: {:?} gets: {:?}, returns {:?}", std::str::from_utf8(name.as_ref()) , A::get_types(),R::get_types());
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R> {
            println!("function mut: {:?} gets: {:?}, returns {:?}", std::str::from_utf8(name.as_ref()) , A::get_types(),R::get_types());
    }

    fn add_meta_method<A, R, M>(&mut self, _: MetaMethod, _: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R> {
        
    }

    fn add_meta_method_mut<A, R, M>(&mut self, _: MetaMethod, _: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R> {
    
    }

    fn add_meta_function<A, R, F>(&mut self, _: MetaMethod, _: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R> {
        
    }

    fn add_meta_function_mut<A, R, F>(&mut self, _: MetaMethod, _: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R> {
        
    }
}