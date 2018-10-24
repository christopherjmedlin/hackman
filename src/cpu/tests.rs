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

#[test]
fn test_indirect_loads() {
    let mut cpu = Z80::new();
    let mut mem = TestMemory::new();
    cpu.reg.write_bc(0x0012);
    mem.ram[0x0012] = 0x11;
    
    cpu.run_opcode(0x0A, &mut mem);

    assert_eq!(cpu.reg.a, mem.ram[0x0012]);
}

#[test]
fn test_16bit_inc_dec() {
    let mut cpu = Z80::new();
    let mut mem = TestMemory::new();

    cpu.run_opcode(0x33, &mut mem);
    assert_eq!(cpu.reg.sp, 1);
    cpu.run_opcode(0x3B, &mut mem);
    assert_eq!(cpu.reg.sp, 0);
}

#[test]
fn test_8bit_inc_dec() {
    let mut cpu = Z80::new();
    let mut mem = TestMemory::new();

    cpu.run_opcode(0x04, &mut mem);
    assert_eq!(cpu.reg.b, 1);
    cpu.run_opcode(0x05, &mut mem);
    assert_eq!(cpu.reg.b, 0);
}

#[test]
fn test_acc_shift() {
    let mut cpu = Z80::new();

    cpu.reg.a = 0b0000_0101;
    cpu.acc_shift(true, false);
    assert_eq!(cpu.reg.a, 0b0000_1010);
    assert_eq!(cpu.reg.cc(3), false);

    cpu.reg.a = 0b0101_0101;
    cpu.acc_shift(false, false);
    assert_eq!(cpu.reg.a, 0b1010_1010);
    assert_eq!(cpu.reg.cc(3), true);
    
    cpu.reg.set_flag(0, false);
    cpu.reg.a = 0b0000_0110;
    cpu.acc_shift(true, true);
    assert_eq!(cpu.reg.a, 0b1000_0011);
    assert_eq!(cpu.reg.cc(3), true);
}
