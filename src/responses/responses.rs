use rocket_dyn_templates::Template;
use rocket::response::Redirect;
use crate::db::models::*;

#[derive(Debug, Responder)]
pub enum AnyResponse {
    Template(Template),
    Redirect(Redirect),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Context {
    user: Option<User>,
    page: Option<Page>,
    error: Option<Error>,
    post: Option<Post>,
    posts: Option<Vec<Post>>,
    space: Option<Space>,
    spaces: Option<Vec<Space>>,
}


impl Context {
    pub fn new() -> Context {
        Context {
            user: None,
            page: None,
            error: None,
            post: None,
            posts: None,
            space: None,
            spaces: None,
        }
    }

    pub fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }

    pub fn set_page(&mut self, page: Page) {
        self.page = Some(page);
    }

    pub fn set_error(&mut self, error: Error) {
        self.error = Some(error);
    }

    pub fn set_post(&mut self, post: Post) {
        self.post = Some(post);
    }

    pub fn set_posts(&mut self, posts: Vec<Post>) {
        self.posts = Some(posts);
    }

    pub fn set_space(&mut self, space: Space) {
        self.space = Some(space);
    }

    pub fn set_spaces(&mut self, spaces: Vec<Space>) {
        self.spaces = Some(spaces);
    }
}
