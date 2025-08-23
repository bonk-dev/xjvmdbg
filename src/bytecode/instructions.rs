enum Instruction {
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
        default: i32,
        npairs: i32,
        matches: Vec<(i32, i32)>,
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
}

enum WideInstruction {
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
