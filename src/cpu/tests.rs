use super::*;
use cpu::mem::TestMemory;
use cpu::io::TestIO;

#[test]
fn test_run_opcodes() {
    let mut cpu = Z80::new();
    let mut mem = TestMemory::new();
    let mut io = TestIO::new();

    mem.ram[0] = 0x00;
    mem.ram[1] = 0x00;
    mem.ram[2] = 0x00;
    cpu.run_opcodes(3, &mut mem, &mut io);

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
    let mut io = TestIO::new();

    mem.ram[2] = 0x18;
    mem.ram[3] = 0x05;
    cpu.run_opcodes(1, &mut mem, &mut io);
    assert_eq!(cpu.reg.pc, 7);
}

#[test]
fn test_indirect_loads() {
    let mut cpu = Z80::new();
    let mut mem = TestMemory::new();
    let mut io = TestIO::new();

    cpu.reg.write_bc(0x0012);
    mem.ram[0x0012] = 0x11;
    
    cpu.run_opcode(0x0A, &mut mem, &mut io);

    assert_eq!(cpu.reg.a, mem.ram[0x0012]);
}

#[test]
fn test_16bit_inc_dec() {
    let mut cpu = Z80::new();
    let mut mem = TestMemory::new();
    let mut io = TestIO::new();

    cpu.run_opcode(0x33, &mut mem, &mut io);
    assert_eq!(cpu.reg.sp, 1);
    cpu.run_opcode(0x3B, &mut mem, &mut io);
    assert_eq!(cpu.reg.sp, 0);
}

#[test]
fn test_8bit_inc_dec() {
    let mut cpu = Z80::new();
    let mut mem = TestMemory::new();
    let mut io = TestIO::new();

    cpu.run_opcode(0x04, &mut mem, &mut io);
    assert_eq!(cpu.reg.b, 1);
    cpu.run_opcode(0x05, &mut mem, &mut io);
    assert_eq!(cpu.reg.b, 0);
    assert_eq!(cpu.reg.read_flag(6), true);
    assert_eq!(cpu.reg.read_flag(1), true);

    // test overflow
    cpu.reg.b = 127;
    cpu.run_opcode(0x05, &mut mem, &mut io);
    assert_eq!(cpu.reg.read_flag(2), true);
}

#[test]
fn test_acc_shift() {
    let mut cpu = Z80::new();
    let mut mem = &mut TestMemory::new();

    cpu.reg.a = 0b0000_0101;
    cpu.shift(7, true, false, mem);
    assert_eq!(cpu.reg.a, 0b0000_1010);
    assert_eq!(cpu.reg.cc(3), false);

    cpu.reg.a = 0b0101_0101;
    cpu.shift(7, false, false, mem);
    assert_eq!(cpu.reg.a, 0b1010_1010);
    assert_eq!(cpu.reg.cc(3), true);
    
    cpu.reg.set_flag(0, false);
    cpu.reg.a = 0b0000_0110;
    cpu.shift(7, true, true, mem);
    assert_eq!(cpu.reg.a, 0b1000_0011);
    assert_eq!(cpu.reg.cc(3), true);
}

#[test]
fn test_detect_overflow() {
    let mut cpu = Z80::new();
    
    cpu.detect_overflow_add(70, 70, false);
    assert_eq!(cpu.reg.read_flag(2), true);
    cpu.detect_overflow_add(10, 10, false);
    assert_eq!(cpu.reg.read_flag(2), false);

    cpu.detect_overflow_sub(70, 186, false);
    assert_eq!(cpu.reg.read_flag(2), true);
    cpu.detect_overflow_sub(70, 70, false);
    assert_eq!(cpu.reg.read_flag(2), false);
}

#[test]
fn test_halt() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();

    cpu.run_opcode(0x76, &mut memory, &mut io);
    assert_eq!(cpu.run_opcodes(100, &mut memory, &mut io), 0);
}

#[test]
fn test_8bit_loading() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();
    
    cpu.reg.a = 5;
    cpu.run_opcode(0x47, &mut memory, &mut io);
    assert_eq!(cpu.reg.b, 5);
}

#[test]
fn test_add() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();

    cpu.reg.a = 5;
    cpu.reg.b = 6;
    cpu.run_opcode(0x80, &mut memory, &mut io);
    assert_eq!(cpu.reg.a, 11);

    cpu.reg.set_flag(0, true);
    cpu.run_opcode(0x88, &mut memory, &mut io);
    // 11 + 6 + 1 (carry flag) = 18
    assert_eq!(cpu.reg.a, 18);
}

#[test]
fn test_sub() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();

    cpu.reg.a = 5;
    cpu.reg.b = 2;
    cpu.run_opcode(0x90, &mut memory, &mut io);
    assert_eq!(cpu.reg.a, 3);

    cpu.reg.set_flag(0, true);
    cpu.run_opcode(0x98, &mut memory, &mut io);
    assert_eq!(cpu.reg.a, 0);
}

#[test]
fn test_stack() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();

    cpu.reg.sp = 50;
    cpu.push_stack(&mut memory, 10);
    cpu.push_stack(&mut memory, 6);
    cpu.push_stack(&mut memory, 5);
    assert_eq!(memory.ram[49], 10);
    
    assert_eq!(cpu.pop_stack(&mut memory), 5);
    assert_eq!(cpu.pop_stack(&mut memory), 6);
    assert_eq!(cpu.pop_stack(&mut memory), 10);
    assert_eq!(cpu.reg.sp, 50);

    cpu.push_stack_16(&mut memory, 500);
    cpu.push_stack_16(&mut memory, 600);
    assert_eq!(cpu.reg.sp, 46);
    assert_eq!(cpu.pop_stack_16(&mut memory), 600);
    assert_eq!(cpu.pop_stack_16(&mut memory), 500);
}

#[test]
fn test_return() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    
    cpu.reg.pc = 100;
    cpu.reg.sp = 50;
    cpu.call(&mut memory, 150);
    assert_eq!(cpu.reg.pc, 150);
    cpu.ret(&mut memory);
    assert_eq!(cpu.reg.pc, 103);
    assert_eq!(cpu.reg.sp, 50);
}

#[test]
fn test_bit_test() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();

    cpu.reg.b = 2;
    memory.ram[0] = 0xCB;
    memory.ram[1] = 0x48;
    cpu.run_opcode(memory.read_byte(0), &mut memory, &mut io);
    assert_eq!(cpu.reg.read_flag(6), true);
}

#[test]
fn test_16bit_ld() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();
    
    cpu.reg.write_hl(0x1337);
    memory.ram[0] = 0xED;
    memory.ram[1] = 0x63;
    memory.ram[2] = 0x00;
    memory.ram[3] = 0x20;
    cpu.run_opcode(memory.read_byte(0), &mut memory, &mut io);
    assert_eq!(memory.read_word(0x0020), 0x1337);
}

#[test]
fn test_neg() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();
    
    cpu.reg.a = 8;
    memory.ram[0] = 0xED;
    memory.ram[1] = 0x4C;
    cpu.run_opcode(memory.read_byte(0), &mut memory, &mut io);

    assert_eq!(cpu.reg.a as i8, -8);
}

#[test]
fn test_interrupt() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();
    
    memory.ram[771] = 0x09;
    cpu.interrupt(2);
    cpu.reg.i = 3;
    cpu.reg.sp = 100;;
    cpu.run_opcode(0x00, &mut memory, &mut io);
    
    // taking into account the PC increment
    assert_eq!(cpu.reg.pc, 0x0A);
}

#[test]
fn test_io() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();
    
    io.data = 20;
    cpu.reg.d = 1;
    memory.ram[0] = 0xED;
    memory.ram[1] = 0x40;
    memory.ram[2] = 0xED;
    memory.ram[3] = 0x51;
    cpu.run_opcode(memory.read_byte(0), &mut memory, &mut io);
    assert_eq!(cpu.reg.b, 20);
    cpu.run_opcode(memory.read_byte(2), &mut memory, &mut io);
    assert_eq!(io.data, 1);
}

#[test]
fn test_ix_operations() {
    let mut cpu = Z80::new();
    let mut memory = TestMemory::new();
    let mut io = TestIO::new();

    memory.ram[0] = 0xDD;
    memory.ram[1] = 0x21;
    memory.ram[2] = 0x13;
    memory.ram[3] = 0x37;
    memory.ram[4] = 0xDD;
    memory.ram[5] = 0x39;
    memory.ram[6] = 0x21;
    memory.ram[7] = 0x11;
    memory.ram[8] = 0x11;
    // LD IX, **
    cpu.run_opcode(memory.read_byte(0), &mut memory, &mut io);
    cpu.reg.patch_ix(true);
    assert_eq!(cpu.reg.hl(), 0x1337);
    assert_eq!(cpu.reg.pc, 4);
    cpu.reg.patch_ix(false);
    cpu.reg.sp = 0x1000;
    // ADD IX, SP
    cpu.run_opcode(memory.read_byte(4), &mut memory, &mut io);
    cpu.reg.patch_ix(true);
    assert_eq!(cpu.reg.hl(), 0x2337);
    assert_eq!(cpu.reg.pc, 6);
    cpu.reg.patch_ix(false);
    // test a regular HL opcode just incase
    cpu.run_opcode(memory.read_byte(6), &mut memory, &mut io);
    assert_eq!(cpu.reg.hl(), 0x1111);
    cpu.reg.patch_ix(true);
    assert_eq!(cpu.reg.hl(), 0x2337);
}
