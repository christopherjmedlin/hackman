use super::*;
use cpu::mem::TestMemory;

#[test]
fn test_run_opcodes() {
    let mut cpu = Z80::new();
    let mut mem = TestMemory::new();
    mem.ram[0] = 0x00;
    mem.ram[1] = 0x00;
    mem.ram[2] = 0x00;
    cpu.run_opcodes(3, &mut mem);

    assert_eq!(cpu.reg.pc, 3);
}

#[test]
fn test_jr() {
    let mut cpu = Z80::new();
    cpu.jr(5);
    assert_eq!(cpu.reg.pc, 5);
    cpu.jr(-3);
    assert_eq!(cpu.reg.pc, 2);
    
    let mut mem = TestMemory::new();
    mem.ram[2] = 0x18;
    mem.ram[3] = 0x05;
    cpu.run_opcodes(1, &mut mem);
    assert_eq!(cpu.reg.pc, 7);
}
