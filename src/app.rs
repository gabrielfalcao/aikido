use toolz::ironpunk;

pub fn main() -> Result<(), ironpunk::BoxedError> {
    let routes = ironpunk::BoxedRoutes::new();
    ironpunk::start(routes)
}
