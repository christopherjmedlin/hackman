const GAME_ROM_START: u16 = 0x0000;
const GAME_ROM_END: u16 = 0x3FFF;

const VRAM_TILES_START: u16 = 0x4000;
const VRAM_TILES_END: u16 = 0x43FF;

const VRAM_PALETTES_START: u16 = 0x4400;
const VRAM_PALETTES_END: u16 = 0x47FF;

const RAM_START: u16 = 0x4800;
const RAM_END: u16 = 0x4FEF;

const VRAM_SPRITES_START: u16 = 0x4FF0;
const VRAM_SPRITES_END: u16 = 0x4FFF;

const IN0_REGISTER_START: u16 = 0x5000;
const IN0_REGISTER_END: u16 = 0x503F;

const INTERRUPT_ENABLE_REGISTER: u16 = 0x5000;
const SOUND_ENABLE_REGISTER: u16 = 0x5001;
const AUX_ENABLE_REGISTER: u16 = 0x5002;
const FLIP_SCREEN_REGISTER: u16 = 0x5003;
const PLAYER_ONE_LAMP_REGISTER: u16 = 0x5004;
const PLAYER_TWO_LAMP_REGISTER: u16 = 0x5005;
const COIN_LOCKOUT_REGISTER: u16 = 0x5006;
const COIN_COUNTER_REGISTER: u16 = 0x5007;

const IN1_REGISTER_START: u16 = 0x5040;
const IN1_REGISTER_END: u16 = 0x507F;

const SOUND_START: u16 = 0x5040;
const SOUND_END: u16 = 0x505F;

const SPRITE_XY_START: u16 = 0x5060;
const SPRITE_XY_END: u16 = 0x506F;

const DIP_SWITCH_START: u16 = 0x5080;
const DIP_SWITCH_END: u16 = 0x50BF;

const WATCHDOG_START: u16 = 0x50C0;
const WATCHDOG_END: u16 = 0x50FF;

pub enum Address {
    GameRom(usize),
    VramTiles(usize),
    VramPalettes(usize),
    Ram(usize),
    VramSprites(usize),
    IN0Register,
    InterruptEnable,
    SoundEnable,
    AuxEnable,
    FlipScreenRegister,
    PlayerOneLampRegister,
    PlayerTwoLampRegister,
    CoinLockoutRegister,
    CoinCounterRegister,
    IN1Register,
    Sound(usize),
    SpriteXYRegister(usize),
    DipSwitchRegister,
    WatchdogTimerReset
}

/// Maps a memory address to an Address enum corresponding
/// to the proper device
///
/// If writing is true, it means that the CPU is writing to the
/// specified address, otherwise it is reading from it.
pub fn map_address(addr: u16, writing: bool) -> Result<Address, &'static str> {
    let address = (addr, writing);
    
    match address {
        (GAME_ROM_START...GAME_ROM_END, false) =>
            Ok(Address::GameRom((addr - GAME_ROM_START) as usize)),

        (GAME_ROM_START...GAME_ROM_END, true) =>
            Err("Cannot write to ROM."),

        (VRAM_TILES_START...VRAM_TILES_END, true) =>
            Ok(Address::VramTiles((addr - VRAM_TILES_START) as usize)),

        (VRAM_PALETTES_START...VRAM_PALETTES_END, true) => 
            Ok(Address::VramSprites((addr - VRAM_SPRITES_START) as usize)),

        (RAM_START...RAM_END, _) => 
            Ok(Address::Ram((addr - RAM_START) as usize)),

        (VRAM_SPRITES_START...VRAM_SPRITES_END, _) =>
            Ok(Address::VramSprites((addr - VRAM_TILES_START) as usize)),
        
        (IN0_REGISTER_START...IN0_REGISTER_END, false) =>
            Ok(Address::IN0Register),

        (INTERRUPT_ENABLE_REGISTER, true) => Ok(Address::InterruptEnable),
        (SOUND_ENABLE_REGISTER, true) => Ok(Address::SoundEnable),
        (AUX_ENABLE_REGISTER, true) => Ok(Address::AuxEnable),
        (FLIP_SCREEN_REGISTER, true) => Ok(Address::FlipScreenRegister),
        (PLAYER_ONE_LAMP_REGISTER, true) => Ok(Address::PlayerOneLampRegister),
        (PLAYER_TWO_LAMP_REGISTER, true) => Ok(Address::PlayerTwoLampRegister),
        (COIN_LOCKOUT_REGISTER, true) => Ok(Address::CoinLockoutRegister),
        (COIN_COUNTER_REGISTER, true) => Ok(Address::CoinCounterRegister),
        
        (IN1_REGISTER_START...IN1_REGISTER_END, true) =>
            Ok(Address::IN1Register),

        (SOUND_START...SOUND_END, true) => 
            Ok(Address::Sound((addr - SOUND_START) as usize)),

        (SPRITE_XY_START...SPRITE_XY_END, true) =>
            Ok(Address::SpriteXYRegister((addr - SPRITE_XY_START) as usize)),
        
        (DIP_SWITCH_START...DIP_SWITCH_END, true) =>
            Ok(Address::DipSwitchRegister),

        (WATCHDOG_START...WATCHDOG_END, true) =>
            Ok(Address::WatchdogTimerReset),

        (_, _) => Err("Could not map address")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_map() {
        let addr = map_address(0x5000, false);
        assert!(matches!(addr.unwrap(), Address::IN0Register));

        let addr = map_address(0x4805, true);
        assert!(matches!(addr.unwrap(), Address::Ram(5)));
    }
}
