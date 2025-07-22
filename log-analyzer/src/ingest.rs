use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::sync::mpsc::Sender;

pub async fn tail_file(path: &Path, tx: Sender<String>) -> std::io::Result<()> {
    let mut file = File::open(path).await?;

    // Seek to the end so we only get *new* lines
    file.seek(SeekFrom::End(0)).await?;

    let mut reader = BufReader::new(file);
    let mut buffer = String::new();

    loop {
        buffer.clear();

        let bytes_read = reader.read_line(&mut buffer).await?;

        if bytes_read == 0 {
            // No new line, sleep briefly and try again
            tokio::time::sleep(Duration::from_millis(250)).await;
            continue;
        }

        // Send new line
        if tx.send(buffer.trim_end().to_string()).await.is_err() {
            break; // Receiver was dropped
        }
    }

    Ok(())
}
