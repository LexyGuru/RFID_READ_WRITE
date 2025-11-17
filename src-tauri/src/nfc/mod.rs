pub mod ntag216;
pub mod reader;
pub mod card_trait;

#[cfg(feature = "libnfc")]
pub mod reader_libnfc;

pub use ntag216::Ntag216;
pub use reader::NfcReader;
pub use card_trait::NfcCard;

#[cfg(feature = "libnfc")]
pub use reader_libnfc::LibnfcReader;
#[cfg(feature = "libnfc")]
pub use card_trait::LibnfcCardWrapper;

