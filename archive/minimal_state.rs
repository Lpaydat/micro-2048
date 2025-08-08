use linera_sdk::views::{linera_views, RegisterView, RootView, ViewStorageContext};

#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct Game2048State {
    pub participants_count: RegisterView<u64>,
}