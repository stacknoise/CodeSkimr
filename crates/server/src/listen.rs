//! The accept loop: a cap on open connections and a time limit for the head of a request.
//!
//! `axum::serve` has neither. A client that opens connections and sends nothing, or one byte at a
//! time, would hold the server's tasks and file handles for good (slowloris). Here a connection
//! needs a free slot before it is accepted, and hyper closes a connection whose request head does
//! not arrive in time.

use std::future::{Future, poll_fn};
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::Poll;

use axum::Router;
use axum::extract::Request;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto::Builder;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};
use tokio::time::sleep;
use tower_service::Service;

use crate::consts::{ACCEPT_RETRY, HEADER_READ_TIMEOUT, MAX_CONNECTIONS};

/// Which of two futures finished first.
enum First<A, B> {
    Left(A),
    Right(B),
}

/// Polls `left` first, then `right`, and gives the output of the one that is ready.
async fn first<A, B>(left: &mut A, right: &mut B) -> First<A::Output, B::Output>
where
    A: Future + Unpin,
    B: Future + Unpin,
{
    poll_fn(|cx| {
        if let Poll::Ready(value) = Pin::new(&mut *left).poll(cx) {
            return Poll::Ready(First::Left(value));
        }
        match Pin::new(&mut *right).poll(cx) {
            Poll::Ready(value) => Poll::Ready(First::Right(value)),
            Poll::Pending => Poll::Pending,
        }
    })
    .await
}

/// Answers requests on `listener` until `shutdown` completes, then lets open connections finish.
pub async fn run(
    listener: TcpListener,
    router: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) {
    let slots = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let (stop, _) = watch::channel(());
    let mut shutdown = pin!(shutdown);
    while let Some(stream) = next_connection(&listener, &slots, &mut shutdown).await {
        tokio::spawn(serve_connection(
            stream.0,
            stream.1,
            router.clone(),
            stop.subscribe(),
        ));
    }
    drop(stop);
    // Every connection gives its slot back when it ends, so all slots free means all are done.
    let all = u32::try_from(MAX_CONNECTIONS).unwrap_or(u32::MAX);
    drop(slots.acquire_many(all).await);
}

/// The next connection with the slot it holds, or `None` once `shutdown` has completed.
async fn next_connection(
    listener: &TcpListener,
    slots: &Arc<Semaphore>,
    shutdown: &mut Pin<&mut (impl Future<Output = ()> + Send)>,
) -> Option<(TcpStream, OwnedSemaphorePermit)> {
    loop {
        let mut slot = pin!(Arc::clone(slots).acquire_owned());
        let permit = match first(shutdown, &mut slot).await {
            First::Left(()) => return None,
            First::Right(permit) => permit.ok()?,
        };
        let mut accept = pin!(listener.accept());
        match first(shutdown, &mut accept).await {
            First::Left(()) => return None,
            First::Right(Ok((stream, _))) => return Some((stream, permit)),
            First::Right(Err(_)) => sleep(ACCEPT_RETRY).await,
        }
    }
}

/// Serves one connection until it ends or the server shuts down gracefully. The slot is given
/// back when this function returns.
async fn serve_connection(
    stream: TcpStream,
    slot: OwnedSemaphorePermit,
    router: Router,
    mut stop: watch::Receiver<()>,
) {
    let service = service_fn(move |request: Request<Incoming>| {
        let mut router = router.clone();
        async move { router.call(request).await }
    });
    let mut builder = Builder::new(TokioExecutor::new());
    builder
        .http1()
        .timer(TokioTimer::new())
        .header_read_timeout(HEADER_READ_TIMEOUT);
    let mut connection =
        pin!(builder.serve_connection_with_upgrades(TokioIo::new(stream), service));
    let mut changed = pin!(stop.changed());
    if let First::Right(_) = first(&mut connection, &mut changed).await {
        connection.as_mut().graceful_shutdown();
        drop(connection.await);
    }
    drop(slot);
}
