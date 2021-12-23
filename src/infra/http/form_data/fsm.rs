use rust_fsm::*;

state_machine! {
    derive(Debug,PartialEq)
    pub FormData(End)
    End => {
        Dash => Boundary[EffectAppendBoundary]
    },
    Boundary => {
        Alpha => Boundary[EffectAppendBoundary],
        Dash => Boundary[EffectAppendBoundary],
        Colon => Boundary[EffectAppendBoundary],
        Cr => Cr0
    },
    Cr0(Lf) => Lf0,
    Lf0 => {
        Alpha => HeaderField[EffectAppendHeaderField],
    },
    HeaderField => {
        Alpha => HeaderField[EffectAppendHeaderField],
        Dash => HeaderField[EffectAppendHeaderField],
        Colon => Colon,
    },
    Colon(Blank) => Blank,
    Blank => {
        Alpha => HeaderValue[EffectAppendHeaderValue],
        Dash => HeaderValue[EffectAppendHeaderValue],
        Colon => HeaderValue[EffectAppendHeaderValue],
    },
    HeaderValue => {
        Alpha => HeaderValue[EffectAppendHeaderValue],
        Dash => HeaderValue[EffectAppendHeaderValue],
        Colon => HeaderValue[EffectAppendHeaderValue],
        Blank => HeaderValue[EffectAppendHeaderValue],
        Cr => Cr1[EffectAppendHeader]
    },
    Cr1(Lf) => Lf1,
    Lf1 => {
        Cr => Cr2,
        Alpha => HeaderField[EffectAppendHeaderField],
    },
    Cr2(Lf) => Lf2,
    Lf2 => {
        Alpha => Data[EffectAppendData],
        Dash => Data[EffectAppendData],
        Colon => Data[EffectAppendData],
        Cr => Data[EffectAppendData],
        Lf => Data[EffectAppendData],
        BoundaryLike => DataToBoundary[EffectAppendLikeBoundary],
    },
    Data => {
        Alpha => Data[EffectAppendData],
        Dash => Data[EffectAppendData],
        Colon => Data[EffectAppendData],
        Blank => Data[EffectAppendData],
        Cr => Data[EffectAppendData],
        Lf => Data[EffectAppendData],
        BoundaryLike => DataToBoundary[EffectAppendLikeBoundary],
    },
    DataToBoundary => {
        BoundaryLike => DataToBoundary[EffectAppendLikeBoundary],
        Alpha => Data[EffectAppendData],
        Dash => Data[EffectAppendData],
        Colon => Data[EffectAppendData],
        Blank => Data[EffectAppendData],
        Cr => Data[EffectAppendData],
        Lf => Data[EffectAppendData],
        EndDash => EndDash1[EffectAppendPart],
        Cr => Cr3[EffectAppendPart],
    },
    Cr3(Lf) => Lf3,
    Lf3 => {
        Alpha => HeaderField[EffectAppendHeaderField],
    },
    EndDash1(EndDash) => EndDash2[EffectFormData],
    EndDash2(End) => End,
}
