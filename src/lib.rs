pub mod state {
    use tokio::sync::{mpsc, oneshot, watch};
    type Connector<T> = (mpsc::UnboundedSender<Command<T>>, watch::Receiver<T>);
    pub async fn make<T>() -> Connector<T>
    where
        T: Default + Clone + Send + Sync + 'static,
    {
        let (querier, mut receiver) = mpsc::unbounded_channel();
        let (notifier, watcher) = watch::channel(T::default());

        tokio::spawn(async move {
            let mut value = T::default();
            while let Some(cmd) = receiver.recv().await {
                match cmd {
                    Command::Get { responder } => {
                        let _ = responder.send(value.clone());
                    }
                    Command::Set {
                        value: new_val,
                        responder,
                    } => {
                        value = new_val;
                        let _ = responder.send(true);
                        let _ = notifier.send(value.clone());
                    }
                }
            }
        });

        (querier, watcher)
    }

    pub enum Command<T>
    where
        T: Default + Clone + Send + Sync + 'static,
    {
        Get {
            responder: oneshot::Sender<T>,
        },
        Set {
            value: T,
            responder: oneshot::Sender<bool>,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state() {
        use tokio::sync::oneshot;

        let (sender, _watcher) = state::make::<String>().await;
        let (tx, rx) = oneshot::channel();
        let _ = sender.send(state::Command::Set {
            value: "val".to_string(),
            responder: tx,
        });
        let res = rx.await.unwrap();
        assert_eq!(true, res);

        let (tx, rx) = oneshot::channel();
        let _ = sender.send(state::Command::Get { responder: tx });
        let res = rx.await.unwrap();
        assert_eq!("val".to_owned(), res);
    }
}
