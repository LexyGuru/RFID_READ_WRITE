use pcsc::*;
use std::time::Duration;

pub struct NFCReader {
    ctx: Context,
}

#[derive(Debug)]
pub enum NFCError {
    ContextError(String),
    NoReaderFound,
    CardError(String),
    Timeout,
}

impl NFCReader {
    pub fn new() -> Result<Self, NFCError> {
        let ctx = Context::establish(Scope::User)
            .map_err(|e| NFCError::ContextError(format!("PC/SC context error: {}", e)))?;
        Ok(NFCReader { ctx })
    }

    pub fn list_readers(&self) -> Result<Vec<String>, NFCError> {
        let mut buffer = [0u8; 2048];
        let readers = self.ctx.list_readers(&mut buffer)
            .map_err(|e| NFCError::ContextError(format!("List readers error: {}", e)))?;
        
        Ok(readers
            .map(|r| r.to_string_lossy().to_string())
            .collect())
    }

    pub fn connect(&self, reader_name: Option<&str>) -> Result<Card, NFCError> {
        let readers = self.list_readers()?;
        
        if readers.is_empty() {
            return Err(NFCError::NoReaderFound);
        }

        let reader_name = reader_name
            .map(|s| s.to_string())
            .unwrap_or_else(|| readers[0].clone());

        let mut buffer = [0u8; 2048];
        let reader = self.ctx
            .list_readers(&mut buffer)
            .map_err(|e| NFCError::ContextError(format!("List readers error: {}", e)))?
            .find(|r| r.to_string_lossy() == reader_name)
            .ok_or(NFCError::NoReaderFound)?;

        let card = self.ctx
            .connect(reader, ShareMode::Shared, Protocols::ANY)
            .map_err(|e| NFCError::CardError(format!("Card connect error: {}", e)))?;

        Ok(card)
    }

    pub fn wait_for_card(&self, timeout: Duration) -> Result<Card, NFCError> {
        let readers = self.list_readers()?;
        
        if readers.is_empty() {
            return Err(NFCError::NoReaderFound);
        }

        // Wait for card presence - próbálkozunk kapcsolódással timeout-ig
        let start = std::time::Instant::now();
        loop {
            let mut buffer = [0u8; 2048];
            let reader = self.ctx
                .list_readers(&mut buffer)
                .map_err(|e| NFCError::ContextError(format!("List readers error: {}", e)))?
                .next()
                .ok_or(NFCError::NoReaderFound)?;

            match self.ctx.connect(reader, ShareMode::Shared, Protocols::ANY) {
                Ok(card) => return Ok(card),
                Err(_) => {
                    if start.elapsed() >= timeout {
                        return Err(NFCError::Timeout);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
}

