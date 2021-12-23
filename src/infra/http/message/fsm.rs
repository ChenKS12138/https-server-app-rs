use rust_fsm::*;

state_machine! {
    derive(Debug,PartialEq)
    pub RequestMessage(End)
    End(Alpha) => Method[EffectAppendMethod],
    Method => {
        Alpha => Method[EffectAppendMethod],
        Blank => Blank0
    },
    Blank0(Alpha) => Path[EffectAppendPath],
    Path => {
        Alpha => Path[EffectAppendPath],
        Blank => Blank1
    },
    Blank1(Alpha) => Version[EffectAppendVersion],
    Version => {
        Alpha => Version[EffectAppendVersion],
        Cr => Cr0
    },
    Cr0(Lf) => Lf0,
    Lf0(Alpha) => HeaderField[EffectAppendHeaderField],
    HeaderField => {
        Alpha => HeaderField[EffectAppendHeaderField],
        Colon => Colon0,
    },
    Colon0(Blank) => Blank2,
    Blank2(Alpha) => HeaderValue[EffectAppendHeaderValue],
    HeaderValue => {
        Alpha => HeaderValue[EffectAppendHeaderValue],
        Blank => HeaderValue[EffectAppendHeaderValue],
        Colon => HeaderValue[EffectAppendHeaderValue],
        Cr => Cr1[EffectAppendHeader]
    },
    Cr1(Lf) => Lf1,
    Lf1 => {
        Alpha => HeaderField[EffectAppendHeaderField],
        Cr => Cr2
    },
    Cr2(Lf) => Lf2[EffectCheckEnd],
    Lf2 => {
        Alpha => Body[EffectAppendBody],
        End => End,
    },
    Body => {
        Alpha => Body[EffectAppendBody],
        Blank => Body[EffectAppendBody],
        Colon => Body[EffectAppendBody],
        Cr => Body[EffectAppendBody],
        Lf => Body[EffectAppendBody],
        End => End
    }
}
