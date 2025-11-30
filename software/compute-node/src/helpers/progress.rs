use std::iter::FusedIterator;

use tokio::sync::watch;

pub struct ProgressIter<I, T>
where
    I: Iterator<Item = T>,
{
    inner: I,
    progress_tx: watch::Sender<Option<f32>>,
    total_size: usize,
    current_count: usize,
}

impl<I, T> ProgressIter<I, T>
where
    I: Iterator<Item = T>,
{
    pub fn new(inner: I, progress_tx: watch::Sender<Option<f32>>) -> Self {
        let total_size = inner.size_hint().1.unwrap_or(0);

        let _ = progress_tx.send(Some(0.0));

        ProgressIter {
            inner,
            progress_tx,
            total_size,
            current_count: 0,
        }
    }
}

impl<I, T> Iterator for ProgressIter<I, T>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();

        if let Some(_) = item {
            self.current_count += 1;

            if self.total_size > 0 {
                let progress = self.current_count as f32 / self.total_size as f32;
                let _ = self.progress_tx.send(Some(progress));
            }
        } else {
            let _ = self.progress_tx.send(Some(1.0));
        }

        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, T> FusedIterator for ProgressIter<I, T> where I: Iterator<Item = T> + FusedIterator {}
