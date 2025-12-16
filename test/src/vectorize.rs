use crate::SomeSharedData;
use worker::{console_log, Env, Request, Response, Result, Vectorize};

#[worker::send]
pub async fn describe(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let vectorize = env.get_binding::<Vectorize>("VECTORIZE")?;
    let details = vectorize.describe().await?;
    console_log!("{:?}", details);
    Response::from_json(&details)
}
