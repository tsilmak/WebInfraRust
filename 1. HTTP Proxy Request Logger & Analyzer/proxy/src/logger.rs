use hyper::Request;
use hyper::Body;

pub fn log_request(req: &Request<Body>) {
    println!("{} {}", req.method(), req.uri());
    // Salvar em arquivo com estrutura JSON ou CSV
}
