use hyper::Request;
use hyper::Body;

pub fn is_allowed(req: &Request<Body>) -> bool {
    // Lógica de filtro, regex ou domínio
    true
}
