use myth::{html, Response};
use sailfish::TemplateOnce;

pub(super) async fn get() -> myth::Result<Response> {
    let markup = Template.render_once()?;
    Ok(html(markup))
}

#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
struct Template;
