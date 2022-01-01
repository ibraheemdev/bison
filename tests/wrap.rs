use std::sync::{Arc, Mutex};

use bison::extract::{nest, state};
use bison::{http::Body, wrap_fn, Bison, Context, Handler, Request, Wrap};

#[tokio::test]
async fn wrap() {
    type State = Arc<Mutex<Vec<usize>>>;

    #[derive(Context)]
    struct Push {
        #[cx(state)]
        state: State,
        #[cx(nest)]
        req: Request,
    }

    let state = State::default();

    let bison = Bison::new()
        .inject(state.clone())
        .wrap(wrap_fn!(async |cx: Push, next| {
            cx.state.lock().unwrap().push(3);
            next.call(cx.req).await
        }))
        .wrap(wrap_fn!(async |cx: Push, next| {
            cx.state.lock().unwrap().push(2);
            next.call(cx.req).await
        }))
        .scope("/", |scope| {
            scope
                .wrap(wrap_fn!(async |cx: Push, next| {
                    cx.state.lock().unwrap().push(7);
                    next.call(cx.req).await
                }))
                .wrap(wrap_fn!(async |cx: Push, next| {
                    cx.state.lock().unwrap().push(6);
                    next.call(cx.req).await
                }))
                .get(
                    "/",
                    (|cx: Push| async move {
                        cx.state.lock().unwrap().push(14);
                        "..."
                    })
                    .wrap(wrap_fn!(async |cx: Push, next| {
                        cx.state.lock().unwrap().push(13);
                        next.call(cx.req).await
                    }))
                    .wrap(
                        wrap_fn!(async |cx: Push, next| {
                            cx.state.lock().unwrap().push(12);
                            next.call(cx.req).await
                        })
                        .wrap(wrap_fn!(async |cx: Push, next| {
                            cx.state.lock().unwrap().push(11);
                            next.call(cx.req).await
                        }))
                        .wrap(wrap_fn!(async |cx: Push, next| {
                            cx.state.lock().unwrap().push(10);
                            next.call(cx.req).await
                        })),
                    )
                    .wrap(wrap_fn!(async |cx: Push, next| {
                        cx.state.lock().unwrap().push(9);
                        next.call(cx.req).await
                    }))
                    .wrap(wrap_fn!(async |cx: Push, next| {
                        cx.state.lock().unwrap().push(8);
                        next.call(cx.req).await
                    })),
                )
                .wrap(wrap_fn!(async |cx: Push, next| {
                    cx.state.lock().unwrap().push(5);
                    next.call(cx.req).await
                }))
                .wrap(wrap_fn!(async |cx: Push, next| {
                    cx.state.lock().unwrap().push(4);
                    next.call(cx.req).await
                }))
        })
        .wrap(wrap_fn!(async |cx: Push, next| {
            cx.state.lock().unwrap().push(1);
            next.call(cx.req).await
        }))
        .wrap(wrap_fn!(async |cx: Push, next| {
            cx.state.lock().unwrap().push(0);
            next.call(cx.req).await
        }));

    bison
        .serve_one(
            http::Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(*state.lock().unwrap(), (0..=14).collect::<Vec<_>>());
}
