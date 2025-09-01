use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
};

use crate::bytecode::opcode::Opcode;
use binrw::{BinRead, BinResult};

#[derive(Debug)]
pub enum Instruction {
    Aaload,
    Aastore,
    AconstNull,
    Aload {
        index: u8,
    },
    Anewarray {
        index: u16,
    },
    Areturn,
    Arraylength,
    Astore {
        index: u8,
    },
    Athrow,
    Baload,
    Bastore,
    Bipush {
        byte: i8,
    },
    Caload,
    Castore,
    Checkcast {
        index: u16,
    },
    D2f,
    D2i,
    D2l,
    Dadd,
    Daload,
    Dastore,
    Dcmpg,
    Dcmpl,
    Dconst0,
    Dconst1,
    Ddiv,
    Dload {
        index: u8,
    },
    Dmul,
    Dneg,
    Drem,
    Dreturn,
    Dstore {
        index: u8,
    },
    Dsub,
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    F2d,
    F2i,
    F2l,
    Fadd,
    Faload,
    Fastore,
    Fcmpg,
    Fcmpl,
    Fconst0,
    Fconst1,
    Fconst2,
    Fdiv,
    Fload {
        index: u8,
    },
    Fmul,
    Fneg,
    Frem,
    Freturn,
    Fstore {
        index: u8,
    },
    Fsub,
    Getfield {
        index: u16,
    },
    Getstatic {
        index: u16,
    },
    Goto {
        offset: i16,
    },
    GotoW {
        offset: i32,
    },
    I2b,
    I2c,
    I2d,
    I2f,
    I2l,
    I2s,
    Iadd,
    Iaload,
    Iand,
    Iastore,
    Iconst {
        value: i32,
    },
    Idiv,
    IfAcmpeq {
        offset: i16,
    },
    IfAcmpne {
        offset: i16,
    },
    IfIcmpeq {
        offset: i16,
    },
    IfIcmpne {
        offset: i16,
    },
    IfIcmplt {
        offset: i16,
    },
    IfIcmpge {
        offset: i16,
    },
    IfIcmpgt {
        offset: i16,
    },
    IfIcmple {
        offset: i16,
    },
    Ifeq {
        offset: i16,
    },
    Ifne {
        offset: i16,
    },
    Iflt {
        offset: i16,
    },
    Ifge {
        offset: i16,
    },
    Ifgt {
        offset: i16,
    },
    Ifle {
        offset: i16,
    },
    Ifnonnull {
        offset: i16,
    },
    Ifnull {
        offset: i16,
    },
    Iinc {
        index: u8,
        const_value: i8,
    },
    Iload {
        index: u8,
    },
    Imul,
    Ineg,
    Instanceof {
        index: u16,
    },
    Invokedynamic {
        index: u16,
    },
    Invokeinterface {
        index: u16,
        count: u8,
    },
    Invokespecial {
        index: u16,
    },
    Invokestatic {
        index: u16,
    },
    Invokevirtual {
        index: u16,
    },
    Ior,
    Irem,
    Ireturn,
    Ishl,
    Ishr,
    Istore {
        index: u8,
    },
    Isub,
    Iushr,
    Ixor,
    Jsr {
        offset: i16,
    },
    JsrW {
        offset: i32,
    },
    L2d,
    L2f,
    L2i,
    Ladd,
    Laload,
    Land,
    Lastore,
    Lcmp,
    Lconst0,
    Lconst1,
    Ldc {
        index: u8,
    },
    LdcW {
        index: u16,
    },
    Ldc2W {
        index: u16,
    },
    Ldiv,
    Lload {
        index: u8,
    },
    Lmul,
    Lneg,
    Lookupswitch {
        default_offset: i32,
        matches: HashMap<i32, i32>,
    },
    Lor,
    Lrem,
    Lreturn,
    Lshl,
    Lshr,
    Lstore {
        index: u8,
    },
    Lsub,
    Lushr,
    Lxor,
    Monitorenter,
    Monitorexit,
    Multianewarray {
        index: u16,
        dimensions: u8,
    },
    New {
        index: u16,
    },
    Newarray {
        atype: u8,
    },
    Nop,
    Pop,
    Pop2,
    Putfield {
        index: u16,
    },
    Putstatic {
        index: u16,
    },
    Ret {
        index: u8,
    },
    Return,
    Saload,
    Sastore,
    Sipush {
        short: i16,
    },
    Swap,
    Tableswitch {
        default: i32,
        low: i32,
        high: i32,
        offsets: Vec<i32>,
    },
    Wide(Box<WideInstruction>),

    Unknown {
        error: String,
    },
}

#[derive(Debug)]
pub enum WideInstruction {
    Iload { index: u16 },
    Fload { index: u16 },
    Aload { index: u16 },
    Lload { index: u16 },
    Dload { index: u16 },
    Istore { index: u16 },
    Fstore { index: u16 },
    Astore { index: u16 },
    Lstore { index: u16 },
    Dstore { index: u16 },
    Ret { index: u16 },
    Iinc { index: u16, const_value: i16 },
}

impl BinRead for WideInstruction {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let opcode_raw = u8::read(reader)?;
        let opcode = Opcode::try_from(opcode_raw).map_err(|e| binrw::Error::Custom {
            pos: reader.stream_position().unwrap_or(0),
            err: Box::new(format!("Invalid opcode: 0x{:02X}", e.opcode)),
        })?;

        match opcode {
            Opcode::Iload => Ok(WideInstruction::Iload {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Fload => Ok(WideInstruction::Fload {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Aload => Ok(WideInstruction::Aload {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Lload => Ok(WideInstruction::Lload {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Dload => Ok(WideInstruction::Dload {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Istore => Ok(WideInstruction::Istore {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Fstore => Ok(WideInstruction::Fstore {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Astore => Ok(WideInstruction::Astore {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Lstore => Ok(WideInstruction::Lstore {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Dstore => Ok(WideInstruction::Dstore {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::Ret => Ok(WideInstruction::Ret {
                index: u16::read_options(reader, endian, args)?,
            }),
            Opcode::IInc => Ok(WideInstruction::Iinc {
                index: u16::read_options(reader, endian, args)?,
                const_value: i16::read_options(reader, endian, args)?,
            }),
            other => Err(binrw::Error::Custom {
                pos: reader.stream_position().unwrap_or(0),
                err: Box::new(format!("Invalid wide 0x{:02X}", other as u8)),
            }),
        }
    }
}

fn read_lookup_switch<R: Read + Seek>(reader: &mut R) -> binrw::BinResult<Instruction> {
    let pos = reader.stream_position()?;
    let padding_bytes = (4 - ((pos + 1) % 4)) % 4;
    reader.seek(SeekFrom::Current(padding_bytes as i64))?;

    let default_pos = i32::read_be(reader)?;
    let npairs_count = i32::read_be(reader)?;
    let mut npairs = HashMap::with_capacity(npairs_count as usize);

    for _i in 0..npairs_count {
        let match_i = i32::read_be(reader)?;
        let offset = i32::read_be(reader)?;

        npairs.insert(match_i, offset);
    }

    Ok(Instruction::Lookupswitch {
        default_offset: default_pos,
        matches: npairs,
    })
}

fn read_table_switch<R: Read + Seek>(reader: &mut R) -> binrw::BinResult<Instruction> {
    let pos = reader.stream_position()?;
    let padding_bytes = (4 - ((pos + 1) % 4)) % 4;
    reader.seek(SeekFrom::Current(padding_bytes as i64))?;

    let default = i32::read_be(reader)?;
    let low = i32::read_be(reader)?;
    let high = i32::read_be(reader)?;

    let mut count = high - low + 1;
    if count < 0 {
        count = 0;
    }

    let mut offsets = Vec::with_capacity(count as usize);
    for _i in 0..count {
        offsets.push(i32::read_be(reader)?);
    }

    Ok(Instruction::Tableswitch {
        default: default,
        low: low,
        high: high,
        offsets: offsets,
    })
}

impl BinRead for Instruction {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let opcode_raw = u8::read(reader)?;
        match Opcode::try_from(opcode_raw) {
            Ok(opcode) => {
                let instr = match opcode {
                    Opcode::Aaload => Instruction::Aaload,
                    Opcode::Aastore => Instruction::Aastore,
                    Opcode::AconstNull => Instruction::AconstNull,
                    Opcode::Aload => Instruction::Aload {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Aload0 => Instruction::Aload { index: 0 },
                    Opcode::Aload1 => Instruction::Aload { index: 1 },
                    Opcode::Aload2 => Instruction::Aload { index: 2 },
                    Opcode::Aload3 => Instruction::Aload { index: 3 },
                    Opcode::AnewArray => Instruction::Anewarray {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Areturn => Instruction::Areturn,
                    Opcode::ArrayLength => Instruction::Arraylength,
                    Opcode::Astore => Instruction::Astore {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Astore0 => Instruction::Astore { index: 0 },
                    Opcode::Astore1 => Instruction::Astore { index: 1 },
                    Opcode::Astore2 => Instruction::Astore { index: 2 },
                    Opcode::Astore3 => Instruction::Astore { index: 3 },
                    Opcode::Athrow => Instruction::Athrow,
                    Opcode::Baload => Instruction::Baload,
                    Opcode::Bastore => Instruction::Bastore,
                    Opcode::Bipush => Instruction::Bipush {
                        byte: i8::read_options(reader, endian, args)?,
                    },
                    Opcode::Caload => Instruction::Caload,
                    Opcode::Castore => Instruction::Castore,
                    Opcode::Checkcast => Instruction::Checkcast {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::D2f => Instruction::D2f,
                    Opcode::D2i => Instruction::D2i,
                    Opcode::D2l => Instruction::D2l,
                    Opcode::Dadd => Instruction::Dadd,
                    Opcode::Daload => Instruction::Daload,
                    Opcode::Dastore => Instruction::Dastore,
                    Opcode::Dcmpg => Instruction::Dcmpg,
                    Opcode::Dcmpl => Instruction::Dcmpl,
                    Opcode::Dconst0 => Instruction::Dconst0,
                    Opcode::Dconst1 => Instruction::Dconst1,
                    Opcode::Ddiv => Instruction::Ddiv,
                    Opcode::Dload => Instruction::Dload {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Dload0 => Instruction::Dload { index: 0 },
                    Opcode::Dload1 => Instruction::Dload { index: 1 },
                    Opcode::Dload2 => Instruction::Dload { index: 2 },
                    Opcode::Dload3 => Instruction::Dload { index: 3 },
                    Opcode::Dmul => Instruction::Dmul,
                    Opcode::Dneg => Instruction::Dneg,
                    Opcode::Drem => Instruction::Drem,
                    Opcode::Dreturn => Instruction::Dreturn,
                    Opcode::Dstore => Instruction::Dstore {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Dstore0 => Instruction::Dstore { index: 0 },
                    Opcode::Dstore1 => Instruction::Dstore { index: 1 },
                    Opcode::Dstore2 => Instruction::Dstore { index: 2 },
                    Opcode::Dstore3 => Instruction::Dstore { index: 3 },
                    Opcode::Dsub => Instruction::Dsub,
                    Opcode::Dup => Instruction::Dup,
                    Opcode::DupX1 => Instruction::DupX1,
                    Opcode::DupX2 => Instruction::DupX2,
                    Opcode::Dup2 => Instruction::Dup2,
                    Opcode::Dup2X1 => Instruction::Dup2X1,
                    Opcode::Dup2X2 => Instruction::Dup2X2,
                    Opcode::F2d => Instruction::F2d,
                    Opcode::F2i => Instruction::F2i,
                    Opcode::F2l => Instruction::F2l,
                    Opcode::Fadd => Instruction::Fadd,
                    Opcode::Faload => Instruction::Faload,
                    Opcode::Fastore => Instruction::Fastore,
                    Opcode::Fcmpg => Instruction::Fcmpg,
                    Opcode::Fcmpl => Instruction::Fcmpl,
                    Opcode::Fconst0 => Instruction::Fconst0,
                    Opcode::Fconst1 => Instruction::Fconst1,
                    Opcode::Fconst2 => Instruction::Fconst2,
                    Opcode::Fdiv => Instruction::Fdiv,
                    Opcode::Fload => Instruction::Fload {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Fload0 => Instruction::Fload { index: 0 },
                    Opcode::Fload1 => Instruction::Fload { index: 1 },
                    Opcode::Fload2 => Instruction::Fload { index: 2 },
                    Opcode::Fload3 => Instruction::Fload { index: 3 },
                    Opcode::Fmul => Instruction::Fmul,
                    Opcode::Fneg => Instruction::Fneg,
                    Opcode::Frem => Instruction::Frem,
                    Opcode::Freturn => Instruction::Freturn,
                    Opcode::Fstore => Instruction::Fstore {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Fstore0 => Instruction::Fstore { index: 0 },
                    Opcode::Fstore1 => Instruction::Fstore { index: 1 },
                    Opcode::Fstore2 => Instruction::Fstore { index: 2 },
                    Opcode::Fstore3 => Instruction::Fstore { index: 3 },
                    Opcode::Fsub => Instruction::Fsub,
                    Opcode::Getfield => Instruction::Getfield {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Getstatic => Instruction::Getstatic {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Goto => Instruction::Goto {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Gotow => Instruction::GotoW {
                        offset: i32::read_options(reader, endian, args)?,
                    },
                    Opcode::I2b => Instruction::I2b,
                    Opcode::I2c => Instruction::I2c,
                    Opcode::I2d => Instruction::I2d,
                    Opcode::I2f => Instruction::I2f,
                    Opcode::I2l => Instruction::I2l,
                    Opcode::I2s => Instruction::I2s,
                    Opcode::Iadd => Instruction::Iadd,
                    Opcode::Iaload => Instruction::Iaload,
                    Opcode::Iand => Instruction::Iand,
                    Opcode::Iastore => Instruction::Iastore,
                    Opcode::IconstM1 => Instruction::Iconst { value: -1 },
                    Opcode::Iconst0 => Instruction::Iconst { value: 0 },
                    Opcode::Iconst1 => Instruction::Iconst { value: 1 },
                    Opcode::Iconst2 => Instruction::Iconst { value: 2 },
                    Opcode::Iconst3 => Instruction::Iconst { value: 3 },
                    Opcode::Iconst4 => Instruction::Iconst { value: 4 },
                    Opcode::Iconst5 => Instruction::Iconst { value: 5 },
                    Opcode::Idiv => Instruction::Idiv,
                    Opcode::IfAcmpeq => Instruction::IfAcmpeq {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::IfAcmpne => Instruction::IfAcmpne {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::IfIcmpeq => Instruction::IfIcmpeq {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::IfIcmpne => Instruction::IfIcmpne {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::IfIcmplt => Instruction::IfIcmplt {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::IfIcmpge => Instruction::IfIcmpge {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::IfIcmpgt => Instruction::IfIcmpgt {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::IfIcmple => Instruction::IfIcmple {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ifeq => Instruction::Ifeq {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ifne => Instruction::Ifne {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Iflt => Instruction::Iflt {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ifge => Instruction::Ifge {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ifgt => Instruction::Ifgt {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ifle => Instruction::Ifle {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ifnonnull => Instruction::Ifnonnull {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ifnull => Instruction::Ifnull {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::IInc => Instruction::Iinc {
                        index: u8::read_options(reader, endian, args)?,
                        const_value: i8::read_options(reader, endian, args)?,
                    },
                    Opcode::Iload => Instruction::Iload {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Iload0 => Instruction::Iload { index: 0 },
                    Opcode::Iload1 => Instruction::Iload { index: 1 },
                    Opcode::Iload2 => Instruction::Iload { index: 2 },
                    Opcode::Iload3 => Instruction::Iload { index: 3 },
                    Opcode::Imul => Instruction::Imul,
                    Opcode::Ineg => Instruction::Ineg,
                    Opcode::Instanceof => Instruction::Instanceof {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Invokedynamic => Instruction::Invokedynamic {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Invokeinterface => Instruction::Invokeinterface {
                        index: u16::read_options(reader, endian, args)?,
                        count: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Invokespecial => Instruction::Invokespecial {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Invokestatic => Instruction::Invokestatic {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Invokevirtual => Instruction::Invokevirtual {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ior => Instruction::Ior,
                    Opcode::Irem => Instruction::Irem,
                    Opcode::Ireturn => Instruction::Ireturn,
                    Opcode::Ishl => Instruction::Ishl,
                    Opcode::Ishr => Instruction::Ishr,
                    Opcode::Istore => Instruction::Istore {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Istore0 => Instruction::Istore { index: 0 },
                    Opcode::Istore1 => Instruction::Istore { index: 1 },
                    Opcode::Istore2 => Instruction::Istore { index: 2 },
                    Opcode::Istore3 => Instruction::Istore { index: 3 },
                    Opcode::Isub => Instruction::Isub,
                    Opcode::Iushr => Instruction::Iushr,
                    Opcode::Ixor => Instruction::Ixor,
                    Opcode::Jsr => Instruction::Jsr {
                        offset: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Jsrw => Instruction::JsrW {
                        offset: i32::read_options(reader, endian, args)?,
                    },
                    Opcode::L2d => Instruction::L2d,
                    Opcode::L2f => Instruction::L2f,
                    Opcode::L2i => Instruction::L2i,
                    Opcode::Ladd => Instruction::Ladd,
                    Opcode::Laload => Instruction::Laload,
                    Opcode::Land => Instruction::Land,
                    Opcode::Lastore => Instruction::Lastore,
                    Opcode::Lcmp => Instruction::Lcmp,
                    Opcode::Lconst0 => Instruction::Lconst0,
                    Opcode::Lconst1 => Instruction::Lconst1,
                    Opcode::Ldc => Instruction::Ldc {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Ldcw => Instruction::LdcW {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ldc2w => Instruction::Ldc2W {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ldiv => Instruction::Ldiv,
                    Opcode::Lload => Instruction::Lload {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Lload0 => Instruction::Lload { index: 0 },
                    Opcode::Lload1 => Instruction::Lload { index: 1 },
                    Opcode::Lload2 => Instruction::Lload { index: 2 },
                    Opcode::Lload3 => Instruction::Lload { index: 3 },
                    Opcode::Lmul => Instruction::Lmul,
                    Opcode::Lneg => Instruction::Lneg,
                    Opcode::Lookupswitch => read_lookup_switch(reader)?,
                    Opcode::Lor => Instruction::Lor,
                    Opcode::Lrem => Instruction::Lrem,
                    Opcode::Lreturn => Instruction::Lreturn,
                    Opcode::Lshl => Instruction::Lshl,
                    Opcode::Lshr => Instruction::Lshr,
                    Opcode::Lstore => Instruction::Lstore {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Lstore0 => Instruction::Lstore { index: 0 },
                    Opcode::Lstore1 => Instruction::Lstore { index: 1 },
                    Opcode::Lstore2 => Instruction::Lstore { index: 2 },
                    Opcode::Lstore3 => Instruction::Lstore { index: 3 },
                    Opcode::Lsub => Instruction::Lsub,
                    Opcode::Lushr => Instruction::Lushr,
                    Opcode::Lxor => Instruction::Lxor,
                    Opcode::Monitorenter => Instruction::Monitorenter,
                    Opcode::Monitorexit => Instruction::Monitorexit,
                    Opcode::Multianewarray => Instruction::Multianewarray {
                        index: u16::read_options(reader, endian, args)?,
                        dimensions: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::New => Instruction::New {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Newarray => Instruction::Newarray {
                        atype: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Nop => Instruction::Nop,
                    Opcode::Pop => Instruction::Pop,
                    Opcode::Pop2 => Instruction::Pop2,
                    Opcode::Putfield => Instruction::Putfield {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Putstatic => Instruction::Putstatic {
                        index: u16::read_options(reader, endian, args)?,
                    },
                    Opcode::Ret => Instruction::Ret {
                        index: u8::read_options(reader, endian, args)?,
                    },
                    Opcode::Return => Instruction::Return,
                    Opcode::Saload => Instruction::Saload,
                    Opcode::Sastore => Instruction::Sastore,
                    Opcode::Sipush => Instruction::Sipush {
                        short: i16::read_options(reader, endian, args)?,
                    },
                    Opcode::Swap => Instruction::Swap,
                    Opcode::Tableswitch => read_table_switch(reader)?,
                    Opcode::Wide => Instruction::Wide(Box::new(WideInstruction::read_options(
                        reader, endian, args,
                    )?)),
                };

                Ok(instr)
            }
            Err(e) => Ok(Instruction::Unknown {
                error: format!("Invalid opcode: {}", e.opcode),
            }),
        }
    }
}

pub fn parse_instructions<R: Read + Seek>(r: &mut R) -> BinResult<Vec<Instruction>> {
    let mut current_pos = r.stream_position().map_err(|e| binrw::Error::Custom {
        pos: 0,
        err: Box::new(format!("Could not read stream position, {}", e)),
    })?;
    let end_pos = r.seek(SeekFrom::End(0)).map_err(|e| binrw::Error::Custom {
        pos: 0,
        err: Box::new(format!("Could not seek to end, {}", e)),
    })?;

    r.seek(SeekFrom::Start(current_pos))
        .map_err(|e| binrw::Error::Custom {
            pos: 0,
            err: Box::new(format!("Could not seek to last position, {}", e)),
        })?;

    let mut instructions = vec![];
    while (current_pos) < end_pos {
        let instr_result = Instruction::read_be(r);
        match instr_result {
            Ok(i) => {
                instructions.push(i);
            }
            Err(e) => {
                instructions.push(Instruction::Unknown {
                    error: format!("Could not read instruction: {}", e),
                });
            }
        }

        current_pos = r.stream_position().map_err(|e| binrw::Error::Custom {
            pos: 0,
            err: Box::new(format!("Could not read stream position, {}", e)),
        })?;
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_table_switch_no_padding() {
        // Position 3 (after opcode), no padding needed (3+1 = 4, divisible by 4)
        let data = vec![
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x10, // default = 16
            0x00, 0x00, 0x00, 0x01, // low = 1
            0x00, 0x00, 0x00, 0x03, // high = 3
            0x00, 0x00, 0x00, 0x20, // offset[0] = 32
            0x00, 0x00, 0x00, 0x30, // offset[1] = 48
            0x00, 0x00, 0x00, 0x40, // offset[2] = 64
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(3); // Simulate position after reading opcode

        let result = read_table_switch(&mut cursor).unwrap();

        if let Instruction::Tableswitch {
            default,
            low,
            high,
            offsets,
        } = result
        {
            assert_eq!(default, 16);
            assert_eq!(low, 1);
            assert_eq!(high, 3);
            assert_eq!(offsets, vec![32, 48, 64]);
        } else {
            panic!("Expected Tableswitch instruction");
        }
    }

    #[test]
    fn test_table_switch_with_1_padding() {
        // Position 2 (after opcode), needs 1 padding byte (2+1 = 3, need 1 byte to reach 4)
        let data = vec![
            0xFF, 0xFF, 0x00, // padding
            0x00, 0x00, 0x00, 0x10, // default = 16
            0x00, 0x00, 0x00, 0x02, // low = 2
            0x00, 0x00, 0x00, 0x02, // high = 2 (single case)
            0x00, 0x00, 0x00, 0x25, // offset[0] = 37
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(2); // Simulate position after reading opcode

        let result = read_table_switch(&mut cursor).unwrap();

        if let Instruction::Tableswitch {
            default,
            low,
            high,
            offsets,
        } = result
        {
            assert_eq!(default, 16);
            assert_eq!(low, 2);
            assert_eq!(high, 2);
            assert_eq!(offsets, vec![37]);
        } else {
            panic!("Expected Tableswitch instruction");
        }
    }

    #[test]
    fn test_table_switch_with_2_padding() {
        // Position 1 (after opcode), needs 2 padding bytes (1+1 = 2, need 2 bytes to reach 4)
        let data = vec![
            0xFF, 0x00, 0x00, // padding
            0x00, 0x00, 0x00, 0x05, // default = 5
            0xFF, 0xFF, 0xFF, 0xFF, // low = -1
            0x00, 0x00, 0x00, 0x01, // high = 1
            0x00, 0x00, 0x00, 0x10, // offset[0] = 16 (for -1)
            0x00, 0x00, 0x00, 0x20, // offset[1] = 32 (for 0)
            0x00, 0x00, 0x00, 0x30, // offset[2] = 48 (for 1)
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(1); // Simulate position after reading opcode

        let result = read_table_switch(&mut cursor).unwrap();

        if let Instruction::Tableswitch {
            default,
            low,
            high,
            offsets,
        } = result
        {
            assert_eq!(default, 5);
            assert_eq!(low, -1);
            assert_eq!(high, 1);
            assert_eq!(offsets, vec![16, 32, 48]);
        } else {
            panic!("Expected Tableswitch instruction");
        }
    }

    #[test]
    fn test_table_switch_with_3_padding() {
        // Position 0 (after opcode), needs 3 padding bytes (0+1 = 1, need 3 bytes to reach 4)
        let data = vec![
            0x00, 0x00, 0x00, // padding
            0x00, 0x00, 0x00, 0x00, // default = 0
            0x00, 0x00, 0x00, 0x05, // low = 5
            0x00, 0x00, 0x00, 0x07, // high = 7
            0x00, 0x00, 0x00, 0x15, // offset[0] = 21 (for 5)
            0x00, 0x00, 0x00, 0x25, // offset[1] = 37 (for 6)
            0x00, 0x00, 0x00, 0x35, // offset[2] = 53 (for 7)
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(0); // Simulate position after reading opcode

        let result = read_table_switch(&mut cursor).unwrap();

        if let Instruction::Tableswitch {
            default,
            low,
            high,
            offsets,
        } = result
        {
            assert_eq!(default, 0);
            assert_eq!(low, 5);
            assert_eq!(high, 7);
            assert_eq!(offsets, vec![21, 37, 53]);
        } else {
            panic!("Expected Tableswitch instruction");
        }
    }

    #[test]
    fn test_table_switch_empty_range() {
        // Test edge case where high < low (should result in negative count)
        // This might be invalid bytecode, but we should handle it gracefully
        let data = vec![
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x10, // default = 16
            0x00, 0x00, 0x00, 0x05, // low = 5
            0x00, 0x00, 0x00, 0x03, // high = 3 (< low)
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(3); // No padding needed

        let result = read_table_switch(&mut cursor).unwrap();

        if let Instruction::Tableswitch {
            default,
            low,
            high,
            offsets,
        } = result
        {
            assert_eq!(default, 16);
            assert_eq!(low, 5);
            assert_eq!(high, 3);
            assert_eq!(offsets, vec![]); // Should be empty since count would be negative
        } else {
            panic!("Expected Tableswitch instruction");
        }
    }

    #[test]
    fn test_lookup_switch_no_padding() {
        // Position 3 (after opcode), no padding needed
        let data = vec![
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x10, // default = 16
            0x00, 0x00, 0x00, 0x03, // npairs = 3
            // Pair 1: match=5, offset=20
            0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x14,
            // Pair 2: match=10, offset=30
            0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x1E,
            // Pair 3: match=15, offset=40
            0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x28,
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(3); // No padding needed

        let result = read_lookup_switch(&mut cursor).unwrap();

        if let Instruction::Lookupswitch {
            default_offset,
            matches,
        } = result
        {
            assert_eq!(default_offset, 16);
            assert_eq!(matches.len(), 3);
            assert_eq!(matches.get(&5), Some(&20));
            assert_eq!(matches.get(&10), Some(&30));
            assert_eq!(matches.get(&15), Some(&40));
        } else {
            panic!("Expected Lookupswitch instruction");
        }
    }

    #[test]
    fn test_lookup_switch_with_padding() {
        // Position 1 (after opcode), needs 2 padding bytes
        let data = vec![
            0xFF, 0x00, 0x00, // padding
            0xFF, 0xFF, 0xFF, 0xF0, // default = -16
            0x00, 0x00, 0x00, 0x02, // npairs = 2
            // Pair 1: match=-5, offset=100
            0xFF, 0xFF, 0xFF, 0xFB, 0x00, 0x00, 0x00, 0x64,
            // Pair 2: match=1000, offset=-50
            0x00, 0x00, 0x03, 0xE8, 0xFF, 0xFF, 0xFF, 0xCE,
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(1); // Needs 2 bytes padding

        let result = read_lookup_switch(&mut cursor).unwrap();

        if let Instruction::Lookupswitch {
            default_offset,
            matches,
        } = result
        {
            assert_eq!(default_offset, -16);
            assert_eq!(matches.len(), 2);
            assert_eq!(matches.get(&-5), Some(&100));
            assert_eq!(matches.get(&1000), Some(&-50));
        } else {
            panic!("Expected Lookupswitch instruction");
        }
    }

    #[test]
    fn test_lookup_switch_zero_pairs() {
        // Test with 0 pairs
        let data = vec![
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x08, // default = 8
            0x00, 0x00, 0x00, 0x00, // npairs = 0
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(3); // No padding needed

        let result = read_lookup_switch(&mut cursor).unwrap();

        if let Instruction::Lookupswitch {
            default_offset,
            matches,
        } = result
        {
            assert_eq!(default_offset, 8);
            assert_eq!(matches.len(), 0);
            assert!(matches.is_empty());
        } else {
            panic!("Expected Lookupswitch instruction");
        }
    }

    #[test]
    fn test_lookup_switch_duplicate_keys() {
        // Test that duplicate keys overwrite (HashMap behavior)
        // Note: This would be invalid bytecode, but we should handle it
        let data = vec![
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00,
            0x00, // some data to skip padding + default = 0
            0x00, 0x00, 0x00, 0x02, // npairs = 2
            // Pair 1: match=5, offset=20
            0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x14,
            // Pair 2: match=5, offset=30 (duplicate key)
            0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x1E,
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(3); // No padding needed

        let result = read_lookup_switch(&mut cursor).unwrap();

        if let Instruction::Lookupswitch {
            default_offset,
            matches,
        } = result
        {
            assert_eq!(default_offset, 0);
            assert_eq!(matches.len(), 1); // Only one entry due to duplicate
            assert_eq!(matches.get(&5), Some(&30)); // Last value wins
        } else {
            panic!("Expected Lookupswitch instruction");
        }
    }

    #[test]
    fn test_table_switch_insufficient_data() {
        // Test with insufficient data (should fail)
        let data = vec![
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x10, // default = 16
            0x00, 0x00, 0x00, 0x01, // low = 1
            0x00, 0x00, 0x00, 0x03, // high = 3 (expects 3 offsets)
            0x00, 0x00, 0x00,
            0x20, // offset[0] = 32
                  // Missing offset[1] and offset[2]
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(3);

        let result = read_table_switch(&mut cursor);
        assert!(result.is_err(), "Should fail with insufficient data");
    }

    #[test]
    fn test_lookup_switch_insufficient_data() {
        // Test with insufficient data for pairs
        let data = vec![
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, // default = 0
            0x00, 0x00, 0x00, 0x02, // npairs = 2
            0x00, 0x00, 0x00,
            0x05, // match = 5
                  // Missing offset for first pair and entire second pair
        ];

        let mut cursor = Cursor::new(data);
        cursor.set_position(3);

        let result = read_lookup_switch(&mut cursor);
        assert!(result.is_err(), "Should fail with insufficient data");
    }

    // Helper test to verify padding calculation
    #[test]
    fn test_padding_calculation() {
        // Test the padding formula: (4 - ((pos + 1) % 4)) % 4
        assert_eq!((4 - ((0 + 1) % 4)) % 4, 3); // pos=0 -> 3 padding bytes
        assert_eq!((4 - ((1 + 1) % 4)) % 4, 2); // pos=1 -> 2 padding bytes
        assert_eq!((4 - ((2 + 1) % 4)) % 4, 1); // pos=2 -> 1 padding byte
        assert_eq!((4 - ((3 + 1) % 4)) % 4, 0); // pos=3 -> 0 padding bytes
        assert_eq!((4 - ((4 + 1) % 4)) % 4, 3); // pos=4 -> 3 padding bytes (cycle repeats)
    }
}
