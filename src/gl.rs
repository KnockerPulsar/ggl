use glow::Context;
use std::sync::Arc;

pub type GlContext = Arc<Context>;
static mut GL_CONTEXT: Option<GlContext> = None;

pub fn get_gl() -> &'static mut GlContext {
    unsafe { GL_CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

pub fn set_gl(gl_rc: Arc<Context>) -> &'static mut GlContext {
    unsafe {
        GL_CONTEXT = Some(gl_rc);
    }
    get_gl()
}
