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
    pub fn registered_patterns(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        for route in &self.routes {
            result.push(route.borrow().path());
        }
        result
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
    fn path(&self) -> String {
        String::from("*")
    }
    #[allow(unused_variables)]
    fn matches_path(&self, path: String) -> bool {
        true
    }
    fn render(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<(), Error> {
        let location = context.borrow().location.clone();
        for route in self.routes.iter_mut() {
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

        let has_error = context.borrow_mut().error.exists();
        if !has_error {
            let patterns = self.registered_patterns();
            let (title, message) = if patterns.len() > 0 {
                (
                    format!("Error 404"),
                    format!("route not found: {}", location),
                )
            } else {
                (format!("Error 500"), format!("no routes declared"))
            };
            context
                .borrow_mut()
                .error
                .set_error(Error::with_message(message));
            context.borrow_mut().error.set_title(title);
        }
        let result = context.borrow_mut().error.render(terminal, context.clone());
        result
    }
}
