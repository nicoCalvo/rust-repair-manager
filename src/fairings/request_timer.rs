#![allow(dead_code)]

use std::time::SystemTime;
use rocket::{debug, error, info};

use rocket::{Request, Data, Response};
use rocket::fairing::{Fairing, Info, Kind};




/// Fairing for timing requests.
pub struct RequestTimer;

/// Value stored in request-local state.
#[derive(Copy, Clone)]
struct TimerStart(Option<SystemTime>);


#[rocket::async_trait]
impl Fairing for RequestTimer {
    // This is a request and response fairing named "GET/POST Counter".
    fn info(&self) -> Info {
        Info {
            name: "Authentication fairing",
            kind: Kind::Request | Kind::Response
        }
    }

  
    // Checks auth cookie is provided and it's valid
    async fn on_request(&self, _req: &mut Request<'_>, _data: &mut Data<'_>) {

        let cookies = _req.cookies();
        let auth_cookie = cookies.get_private("user_id");
        _req.local_cache(|| TimerStart(Some(SystemTime::now())));
       
        

        // match request.method() {
        //     Method::Get => self.get.fetch_add(1, Ordering::Relaxed),
        //     Method::Post => self.post.fetch_add(1, Ordering::Relaxed),
        //     _ => return
        // };
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, _res: &mut Response<'r>) {
        // let uri = _req.uri().path().as_str();
        let start_time = _req.local_cache(|| TimerStart(None));
        if let Some(Ok(duration)) = start_time.0.map(|st| st.elapsed()) {
            let ms = duration.as_secs() * 1000 + duration.subsec_millis() as u64;
            debug!("Elapsed time: {}", ms);
            
            _res.set_raw_header("X-Response-Time", format!("{} ms", ms));
        }

   
    }


    //     // Rewrite the response to return the current counts.
    //     if request.method() == Method::Get && request.uri().path() == "/counts" {
    //         let get_count = self.get.load(Ordering::Relaxed);
    //         let post_count = self.post.load(Ordering::Relaxed);
    //         let body = format!("Get: {}\nPost: {}", get_count, post_count);

    //         response.set_status(Status::Ok);
    //         response.set_header(ContentType::Plain);
    //         response.set_sized_body(Cursor::new(body));
    //     }
    // }
}