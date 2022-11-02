use std::rc::Rc;
use glow::Context;

static mut GL_CONTEXT: Option<Rc<Context>> = None;

pub fn get_gl() -> &'static mut Rc<Context> {
    unsafe { GL_CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

pub fn set_gl(gl_rc: Rc<Context>) -> &'static mut Rc<Context> {
    unsafe { GL_CONTEXT = Some(gl_rc); }
    get_gl()
}
