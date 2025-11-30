use anyhow::Result;

use crate::focus_stacking::FocusStacking;

impl FocusStacking {
    pub async fn run_task(&self, stack_id: &str) -> Result<()> {
        self.set_progress(Some(0.0)).await;
        println!("Executing focus stacking on stack: {}", stack_id);

        for i in 0..=10 {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            let progress = (i as f32) / 10.0;
            self.set_progress(Some(progress)).await;

            println!("Focus stacking progress: {:.0}%", progress * 100.0);
        }
        self.set_progress(Some(1.0)).await;

        Ok(())
    }
}
