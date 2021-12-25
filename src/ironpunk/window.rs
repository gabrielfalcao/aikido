use super::base::*;

use crossterm::event::KeyEvent;

use std::marker::PhantomData;
pub use std::{cell::RefCell, rc::Rc};

use tui::Terminal;
#[allow(dead_code)]
pub struct Window<'a> {
    pub routes: BoxedRoutes,
    pub context: Context<'a>,
    _phantom: PhantomData<&'a Context<'a>>,
}

impl<'a> Window<'a> {
    pub fn from_routes(routes: BoxedRoutes) -> Window<'a> {
        Window {
            routes,
            _phantom: PhantomData,
            context: Context::new("/"),
        }
    }
    pub fn new() -> Window<'a> {
        Window::from_routes(BoxedRoutes::new())
    }
    #[allow(unused_variables)]
    pub fn tick(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<LoopEvent, Error> {
        for route in &mut self.routes.clone() {
            if route
                .borrow()
                .matches_path(context.borrow().location.clone())
            {
                match route.borrow_mut().tick(terminal, context.clone()) {
                    Ok(Propagate) => {
                        // proceed to next route
                        continue;
                    }
                    Ok(Refresh) => {
                        return Ok(Refresh);
                    }
                    Ok(any) => {
                        return Ok(any);
                    }
                    Err(err) => return Err(Error::with_message(format!("{}", err))),
                };
            }
        }
        Ok(Propagate)
    }
}

impl Component for Window<'_> {
    fn name(&self) -> &str {
        "Window"
    }
    fn id(&self) -> String {
        String::from("Window")
    }
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<LoopEvent, Error> {
        for route in self.routes.iter_mut() {
            if route
                .borrow()
                .matches_path(context.borrow().location.clone())
            {
                match route
                    .borrow_mut()
                    .process_keyboard(event, terminal, context.clone())
                {
                    Err(err) => {
                        context.borrow_mut().error.set_error(err);
                        return Ok(Refresh);
                    }
                    ok => return ok,
                }
            }
        }
        if context.borrow_mut().error.exists() {
            context
                .borrow_mut()
                .error
                .process_keyboard(event, terminal, context.clone())?;
        }
        Ok(Propagate)
    }
}
impl Route for Window<'_> {
    #[allow(unused_variables)]
    fn matches_path(&self, path: String) -> bool {
        true
    }
    fn render(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<(), Error> {
        for route in self.routes.iter_mut() {
            let location = context.borrow().location.clone();
            if route.borrow().matches_path(location.clone()) {
                log(format!(
                    "route {} matches {}\n",
                    route.borrow().name(),
                    location
                ));
                route.borrow_mut().render(terminal, context)?;
                return Ok(());
            }
        }

        let has_error = context.borrow().error.exists();
        if !has_error {
            context
                .borrow_mut()
                .error
                .set_error(Error::with_message(format!("no routes declared")));
        }
        let result = context.borrow_mut().error.render(terminal, context.clone());
        result
    }
}
