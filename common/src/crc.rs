use crc::Crc as CrcImpl;
use crc::Digest;
use postcard::flavors::SerFlavor;

pub struct Crc<'a, B: SerFlavor> {
    flav: B,
    digest: Digest<'a, u16>,
}

pub const CRC: CrcImpl<u16> = CrcImpl::<u16>::new(&crc::CRC_16_CDMA2000);

impl<B: SerFlavor> Crc<'_, B> {
    pub fn new(flav: B) -> Self {
        Self {
            flav,
            digest: CRC.digest()
        }
    }
}

impl<B: SerFlavor> SerFlavor for Crc<'_, B> {
    type Output = <B as SerFlavor>::Output;

    fn try_extend(&mut self, data: &[u8]) -> Result<(), ()> {
        self.digest.update(data);
        self.flav.try_extend(data)
    }

    fn try_push(&mut self, data: u8) -> Result<(), ()> {
        self.digest.update(&[data]);
        self.flav.try_push(data)
    }

    fn release(mut self) -> Result<Self::Output, ()> {
        let crc = self.digest.finalize();
        self.flav.try_extend(&crc.to_le_bytes())?;
        self.flav.release()
    }
}