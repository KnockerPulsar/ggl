use std::sync::Arc;
use glow::Context;

static mut GL_CONTEXT: Option<Arc<Context>> = None;

pub fn get_gl() -> &'static mut Arc<Context> {
    unsafe { GL_CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

pub fn set_gl(gl_rc: Arc<Context>) -> &'static mut Arc<Context> {
    unsafe { GL_CONTEXT = Some(gl_rc); }
    get_gl()
}
