use toolz::ironpunk;
use toolz::routes::app::StackedApplication;

pub fn main() -> Result<(), ironpunk::BoxedError> {
    let mut routes = ironpunk::BoxedRoutes::new();
    let main = Box::new(StackedApplication::new("Hello World"));
    routes.push(main);
    ironpunk::start(routes)
}
