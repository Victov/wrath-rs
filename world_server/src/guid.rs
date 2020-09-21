

pub struct Guid(u64);

//from: https://github.com/azerothcore/azerothcore-wotlk/blob/master/src/server/game/Entities/Object/ObjectDefines.h
#[allow(dead_code)]
pub enum HighGuid
{
    Player         = 0x0000, // blizz 0000
    ItemOrContainer= 0x4000, // blizz 4000
    GameObject     = 0xF110, // blizz F110
    Transport      = 0xF120, // blizz F120 (for GAMEOBJECT_TYPE_TRANSPORT)
    Unit           = 0xF130, // blizz F130
    Pet            = 0xF140, // blizz F140
    Vehicle        = 0xF150, // blizz F550
    DynamicObject  = 0xF100, // blizz F100
    Corpse         = 0xF101, // blizz F100
    MoTransport    = 0x1FC0, // blizz 1FC0 (for GAMEOBJECT_TYPE_MO_TRANSPORT)
    Group          = 0x1F50,
    Instance       = 0x1F42,  // blizz 1F42/1F44/1F44/1F47
}

impl Guid
{
    pub fn new(low:u32, mid:u32, high: HighGuid) -> Self
    {
        Self((low as u64) | ((mid as u64) << 24) | ((high as u64) << 48))
    }

    pub fn get_low_part(&self) -> u32
    {
        self.0 as u32
    }
}

pub trait WriteGuid
{
    fn write_guid<T: podio::Endianness>(&mut self, guid: &Guid) -> anyhow::Result<()>;
}

impl<W: std::io::Write> WriteGuid for W
{
    fn write_guid<T: podio::Endianness>(&mut self, guid: &Guid) -> anyhow::Result<()>
    {
        use podio::WritePodExt;
        self.write_u64::<T>(guid.0)?;
        Ok(())
    }
}

pub trait ReadGuid
{
    fn read_guid<T: podio::Endianness>(&mut self) -> anyhow::Result<Guid>;
}

impl<R: std::io::Read> ReadGuid for R
{
    fn read_guid<T: podio::Endianness>(&mut self) -> anyhow::Result<Guid>
    {
        use podio::ReadPodExt;
        let val = self.read_u64::<T>()?;
        Ok(Guid(val))
    }
}
