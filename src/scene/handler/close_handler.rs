use crate::{
    app::CloseRequest,
    event::{EventContext, EventManager},
};

pub struct CloseHandler {
    close_request: Option<CloseRequest>,
    quitting: bool,
}
impl CloseHandler {
    pub fn new() -> Self {
        Self {
            close_request: None,
            quitting: false,
        }
    }

    pub fn handle_close(&mut self, context: &mut EventContext, event_manager: &mut EventManager) {
        if let Some(close_request) = context.close_request {
            self.close_request = Some(close_request);
            event_manager.send_close_request(close_request);
        }
        if let Some(close_request) = self.close_request
            && close_request.requested_at.elapsed() >= close_request.duration
        {
            self.quitting = true;
            event_manager.send_quit();
            for id in context.get_nodes() {
                context.destroy(id);
            }
        }
        if context.is_cancel_close_requested() {
            self.close_request = None;
            event_manager.cancel_close();
        }
    }

    pub fn is_quitting(&self) -> bool {
        self.quitting
    }
}
