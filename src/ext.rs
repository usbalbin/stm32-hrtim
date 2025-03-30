#[cfg(feature = "stm32f3")]
pub use stm32f3::*;
#[cfg(feature = "stm32g4")]
pub use stm32g4::*;
#[cfg(feature = "stm32h7")]
pub use stm32h7::*;

use crate::pac;
use pac::hrtim_master as master;
use pac::hrtim_tima as tima;
use pac::hrtim_timb as timb;
use pac::hrtim_timc as timc;
use pac::hrtim_timd as timd;
use pac::hrtim_time as time;
#[cfg(feature = "hrtim_v2")]
use pac::hrtim_timf as timf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Chan {
    Ch1 = 0,
    Ch2 = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmp {
    Cmp1 = 0,
    Cmp2 = 1,
    Cmp3 = 2,
    Cmp4 = 3,
}

pub trait MasterExt {
    type CRrs: reg::MasterCrR + reg::MasterCrW;
    fn cr(&self) -> &Reg<Self::CRrs>;
    type ISRrs: reg::MasterIsr;
    fn isr(&self) -> &Reg<Self::ISRrs>;
    type ICRrs: reg::MasterIcr;
    fn icr(&self) -> &Reg<Self::ICRrs>;
    type DIERrs: reg::MasterDierR + reg::MasterDierW;
    fn dier(&self) -> &Reg<Self::DIERrs>;
    fn cntr(&self) -> &master::CNTR;
    fn perr(&self) -> &master::PERR;
    fn repr(&self) -> &master::REPR;
    fn cmp1r(&self) -> &master::CMP1R;
    fn cmp2r(&self) -> &master::CMP1R;
    fn cmp3r(&self) -> &master::CMP1R;
    fn cmp4r(&self) -> &master::CMP1R;
    fn cmpr(&self, c: Cmp) -> &master::CMP1R {
        match c {
            Cmp::Cmp1 => self.cmp1r(),
            Cmp::Cmp2 => self.cmp2r(),
            Cmp::Cmp3 => self.cmp3r(),
            Cmp::Cmp4 => self.cmp4r(),
        }
    }
}

pub trait TimExt:
    MasterExt<ISRrs = tima::isr::ISRrs, ICRrs = tima::icr::ICRrs, DIERrs = tima::dier::DIERrs>
{
    fn cmp1cr(&self) -> &tima::CMP1CR;
    fn cpt1r(&self) -> &tima::CPT1R;
    fn cpt2r(&self) -> &tima::CPT1R;
    fn cptr(&self, c: Chan) -> &tima::CPT1R {
        match c {
            Chan::Ch1 => self.cpt1r(),
            Chan::Ch2 => self.cpt2r(),
        }
    }
    fn dtr(&self) -> &tima::DTR;
    fn eefr1(&self) -> &tima::EEFR1;
    fn eefr2(&self) -> &tima::EEFR2;
    type SET1Rrs: RegisterSpec<Ux = u32> + Readable + Writable + Resettable;
    type RST1Rrs: RegisterSpec<Ux = u32> + Readable + Writable + Resettable;
    fn set1r(&self) -> &Reg<Self::SET1Rrs>;
    fn rst1r(&self) -> &Reg<Self::RST1Rrs>;
    fn set2r(&self) -> &Reg<Self::SET1Rrs>;
    fn rst2r(&self) -> &Reg<Self::RST1Rrs>;
    fn set_r(&self, c: Chan) -> &Reg<Self::SET1Rrs> {
        match c {
            Chan::Ch1 => self.set1r(),
            Chan::Ch2 => self.set2r(),
        }
    }
    fn rst_r(&self, c: Chan) -> &Reg<Self::RST1Rrs> {
        match c {
            Chan::Ch1 => self.rst1r(),
            Chan::Ch2 => self.rst2r(),
        }
    }
    type RSTRrs: RegisterSpec<Ux = u32> + Readable + Writable + Resettable;
    fn rstr(&self) -> &Reg<Self::RSTRrs>;
    fn chpr(&self) -> &tima::CHPR;
    type CPT1CRrs: reg::CptcrW + Readable;
    fn cpt1cr(&self) -> &Reg<Self::CPT1CRrs>;
    fn cpt2cr(&self) -> &Reg<Self::CPT1CRrs>;
    fn cptcr(&self, c: Chan) -> &Reg<Self::CPT1CRrs> {
        match c {
            Chan::Ch1 => self.cpt1cr(),
            Chan::Ch2 => self.cpt2cr(),
        }
    }
    fn outr(&self) -> &tima::OUTR;
    fn fltr(&self) -> &tima::FLTR;
    #[cfg(feature = "hrtim_v2")]
    fn cr2(&self) -> &tima::CR2;
    #[cfg(feature = "hrtim_v2")]
    fn eefr3(&self) -> &tima::EEFR3;
}

macro_rules! wrap_r {
    (pub trait $TrR:ident {
        $(fn $f:ident(&self $(, $n:ident: u8)?) -> $fr:path;)*
    }) => {
        pub trait $TrR {
            $(fn $f(&self $(, $n: u8)?) -> $fr;)*
        }
        impl<REG: reg::$TrR> $TrR for R<REG> {
            $(
                #[inline(always)]
                fn $f(&self $(, $n: u8)?) -> $fr {
                    REG::$f(self $(, $n)?)
                }
            )*
        }
    };
}

macro_rules! wrap_w {
    (pub trait $TrR:ident {
        $(fn $f:ident(&mut self $(, $n:ident: u8)?) -> $fr:path;)*
    }) => {
        pub trait $TrR<REG: reg::$TrR> {
            $(fn $f(&mut self $(, $n: u8)?) -> $fr;)*
        }

        impl<REG: reg::$TrR> $TrR<REG> for W<REG> {
            $(
                #[inline(always)]
                fn $f(&mut self $(, $n: u8)?) -> $fr {
                    REG::$f(self $(, $n)?)
                }
            )*
        }
    };
}

wrap_r! {
    pub trait MasterCrR {
        fn ckpsc(&self) -> master::cr::CKPSC_R;
        fn cont(&self) -> master::cr::CONT_R;
        fn retrig(&self) -> master::cr::RETRIG_R;
        fn half(&self) -> master::cr::HALF_R;
        fn syncrst(&self) -> master::cr::SYNCRST_R;
        fn syncstrt(&self) -> master::cr::SYNCSTRT_R;
        fn dacsync(&self) -> master::cr::DACSYNC_R;
        fn preen(&self) -> master::cr::PREEN_R;
    }
}

wrap_w! {
    pub trait MasterCrW {
        fn ckpsc(&mut self) -> master::cr::CKPSC_W<REG>;
        fn cont(&mut self) -> master::cr::CONT_W<REG>;
        fn retrig(&mut self) -> master::cr::RETRIG_W<REG>;
        fn half(&mut self) -> master::cr::HALF_W<REG>;
        fn syncrst(&mut self) -> master::cr::SYNCRST_W<REG>;
        fn syncstrt(&mut self) -> master::cr::SYNCSTRT_W<REG>;
        fn dacsync(&mut self) -> master::cr::DACSYNC_W<REG>;
        fn preen(&mut self) -> master::cr::PREEN_W<REG>;
    }
}

wrap_r! {
pub trait TimCrR {
        fn pshpll(&self) -> tima::cr::PSHPLL_R;
        fn delcmp2(&self) -> tima::cr::DELCMP2_R;
        fn delcmp4(&self) -> tima::cr::DELCMP4_R;
        fn mstu(&self) -> tima::cr::MSTU_R;
        fn trepu(&self) -> tima::cr::TREPU_R;
        fn trstu(&self) -> tima::cr::TRSTU_R;
        fn updgat(&self) -> tima::cr::UPDGAT_R;
    }
}
wrap_w! {
    pub trait TimCrW {
        fn pshpll(&mut self) -> tima::cr::PSHPLL_W<REG>;
        fn delcmp2(&mut self) -> tima::cr::DELCMP2_W<REG>;
        fn delcmp4(&mut self) -> tima::cr::DELCMP4_W<REG>;
        fn mstu(&mut self) -> tima::cr::MSTU_W<REG>;
        fn trepu(&mut self) -> tima::cr::TREPU_W<REG>;
        fn trstu(&mut self) -> tima::cr::TRSTU_W<REG>;
        fn updgat(&mut self) -> tima::cr::UPDGAT_W<REG>;
    }
}

pub trait CptcrW<REG: reg::CptcrW> {
    fn set_swcpt(&mut self) -> &mut Self;
}
impl<REG: reg::CptcrW> CptcrW<REG> for W<REG> {
    #[inline(always)]
    fn set_swcpt(&mut self) -> &mut Self {
        REG::set_swcpt(self)
    }
}

wrap_r! {
    pub trait MasterIsr {
        fn cmp(&self, n: u8) -> master::isr::CMP_R;
        fn cmp1(&self) -> master::isr::CMP_R;
        fn cmp2(&self) -> master::isr::CMP_R;
        fn cmp3(&self) -> master::isr::CMP_R;
        fn cmp4(&self) -> master::isr::CMP_R;
        fn rep(&self) -> master::isr::REP_R;
        fn upd(&self) -> master::isr::UPD_R;
    }
}
wrap_w! {
    pub trait MasterIcr {
        fn cmpc(&mut self, n: u8) -> master::icr::CMPC_W<REG>;
        fn cmp1c(&mut self) -> master::icr::CMPC_W<REG>;
        fn cmp2c(&mut self) -> master::icr::CMPC_W<REG>;
        fn cmp3c(&mut self) -> master::icr::CMPC_W<REG>;
        fn cmp4c(&mut self) -> master::icr::CMPC_W<REG>;
        fn repc(&mut self) -> master::icr::CMPC_W<REG>;
        fn updc(&mut self) -> master::icr::CMPC_W<REG>;
    }
}

wrap_r! {
    pub trait MasterDierR {
        fn cmpie(&self, n: u8) -> master::dier::CMPIE_R;
        fn cmp1ie(&self) -> master::dier::CMPIE_R;
        fn cmp2ie(&self) -> master::dier::CMPIE_R;
        fn cmp3ie(&self) -> master::dier::CMPIE_R;
        fn cmp4ie(&self) -> master::dier::CMPIE_R;
        fn repie(&self) -> master::dier::REPIE_R;
        fn updie(&self) -> master::dier::UPDIE_R;
    }
}

wrap_w! {
    pub trait MasterDierW {
        fn cmpie(&mut self, n: u8) -> master::dier::CMPIE_W<REG>;
        fn cmp1ie(&mut self) -> master::dier::CMPIE_W<REG>;
        fn cmp2ie(&mut self) -> master::dier::CMPIE_W<REG>;
        fn cmp3ie(&mut self) -> master::dier::CMPIE_W<REG>;
        fn cmp4ie(&mut self) -> master::dier::CMPIE_W<REG>;
        fn repie(&mut self) -> master::dier::REPIE_W<REG>;
        fn updie(&mut self) -> master::dier::UPDIE_W<REG>;
    }
}

mod reg {
    use super::*;
    pub trait MasterCrR: RegisterSpec<Ux = u32> + Readable + Sized {
        fn ckpsc(r: &R<Self>) -> master::cr::CKPSC_R;
        fn cont(r: &R<Self>) -> master::cr::CONT_R;
        fn retrig(r: &R<Self>) -> master::cr::RETRIG_R;
        fn half(r: &R<Self>) -> master::cr::HALF_R;
        fn syncrst(r: &R<Self>) -> master::cr::SYNCRST_R;
        fn syncstrt(r: &R<Self>) -> master::cr::SYNCSTRT_R;
        fn dacsync(r: &R<Self>) -> master::cr::DACSYNC_R;
        fn preen(r: &R<Self>) -> master::cr::PREEN_R;
    }
    pub trait MasterCrW: RegisterSpec<Ux = u32> + Writable + Resettable + Sized {
        fn ckpsc(w: &mut W<Self>) -> master::cr::CKPSC_W<Self>;
        fn cont(w: &mut W<Self>) -> master::cr::CONT_W<Self>;
        fn retrig(w: &mut W<Self>) -> master::cr::RETRIG_W<Self>;
        fn half(w: &mut W<Self>) -> master::cr::HALF_W<Self>;
        fn syncrst(w: &mut W<Self>) -> master::cr::SYNCRST_W<Self>;
        fn syncstrt(w: &mut W<Self>) -> master::cr::SYNCSTRT_W<Self>;
        fn dacsync(w: &mut W<Self>) -> master::cr::DACSYNC_W<Self>;
        fn preen(w: &mut W<Self>) -> master::cr::PREEN_W<Self>;
    }

    pub trait TimCrR: MasterCrR {
        fn pshpll(r: &R<Self>) -> tima::cr::PSHPLL_R;
        fn delcmp2(r: &R<Self>) -> tima::cr::DELCMP2_R;
        fn delcmp4(r: &R<Self>) -> tima::cr::DELCMP4_R;
        fn mstu(r: &R<Self>) -> tima::cr::MSTU_R;
        fn trepu(r: &R<Self>) -> tima::cr::TREPU_R;
        fn trstu(r: &R<Self>) -> tima::cr::TRSTU_R;
        fn updgat(r: &R<Self>) -> tima::cr::UPDGAT_R;
    }
    pub trait TimCrW: MasterCrW {
        fn pshpll(w: &mut W<Self>) -> tima::cr::PSHPLL_W<Self>;
        fn delcmp2(w: &mut W<Self>) -> tima::cr::DELCMP2_W<Self>;
        fn delcmp4(w: &mut W<Self>) -> tima::cr::DELCMP4_W<Self>;
        fn mstu(w: &mut W<Self>) -> tima::cr::MSTU_W<Self>;
        fn trepu(w: &mut W<Self>) -> tima::cr::TREPU_W<Self>;
        fn trstu(w: &mut W<Self>) -> tima::cr::TRSTU_W<Self>;
        fn updgat(w: &mut W<Self>) -> tima::cr::UPDGAT_W<Self>;
    }

    pub trait CptcrW: RegisterSpec<Ux = u32> + Writable + Resettable + Sized {
        // TODO: replace this
        fn set_swcpt(w: &mut W<Self>) -> &mut W<Self>;
    }

    pub trait MasterIsr: RegisterSpec<Ux = u32> + Readable + Sized {
        fn cmp(r: &R<Self>, n: u8) -> master::isr::CMP_R;
        fn cmp1(r: &R<Self>) -> master::isr::CMP_R {
            Self::cmp(r, 0)
        }
        fn cmp2(r: &R<Self>) -> master::isr::CMP_R {
            Self::cmp(r, 1)
        }
        fn cmp3(r: &R<Self>) -> master::isr::CMP_R {
            Self::cmp(r, 2)
        }
        fn cmp4(r: &R<Self>) -> master::isr::CMP_R {
            Self::cmp(r, 3)
        }
        fn rep(r: &R<Self>) -> master::isr::REP_R;
        fn upd(r: &R<Self>) -> master::isr::UPD_R;
    }
    pub trait MasterIcr: RegisterSpec<Ux = u32> + Writable + Resettable + Sized {
        fn cmpc(w: &mut W<Self>, n: u8) -> master::icr::CMPC_W<Self>;
        fn cmp1c(w: &mut W<Self>) -> master::icr::CMPC_W<Self> {
            Self::cmpc(w, 0)
        }
        fn cmp2c(w: &mut W<Self>) -> master::icr::CMPC_W<Self> {
            Self::cmpc(w, 1)
        }
        fn cmp3c(w: &mut W<Self>) -> master::icr::CMPC_W<Self> {
            Self::cmpc(w, 2)
        }
        fn cmp4c(w: &mut W<Self>) -> master::icr::CMPC_W<Self> {
            Self::cmpc(w, 3)
        }
        fn repc(w: &mut W<Self>) -> master::icr::CMPC_W<Self>;
        fn updc(w: &mut W<Self>) -> master::icr::CMPC_W<Self>;
    }
    pub trait MasterDierR: RegisterSpec<Ux = u32> + Readable + Sized {
        fn cmpie(r: &R<Self>, n: u8) -> master::dier::CMPIE_R;
        fn cmp1ie(r: &R<Self>) -> master::dier::CMPIE_R {
            Self::cmpie(r, 0)
        }
        fn cmp2ie(r: &R<Self>) -> master::dier::CMPIE_R {
            Self::cmpie(r, 1)
        }
        fn cmp3ie(r: &R<Self>) -> master::dier::CMPIE_R {
            Self::cmpie(r, 2)
        }
        fn cmp4ie(r: &R<Self>) -> master::dier::CMPIE_R {
            Self::cmpie(r, 3)
        }
        fn repie(r: &R<Self>) -> master::dier::REPIE_R;
        fn updie(r: &R<Self>) -> master::dier::UPDIE_R;
    }

    pub trait MasterDierW: RegisterSpec<Ux = u32> + Writable + Resettable + Sized {
        fn cmpie(w: &mut W<Self>, n: u8) -> master::dier::CMPIE_W<Self>;
        fn cmp1ie(w: &mut W<Self>) -> master::dier::CMPIE_W<Self> {
            Self::cmpie(w, 0)
        }
        fn cmp2ie(w: &mut W<Self>) -> master::dier::CMPIE_W<Self> {
            Self::cmpie(w, 1)
        }
        fn cmp3ie(w: &mut W<Self>) -> master::dier::CMPIE_W<Self> {
            Self::cmpie(w, 2)
        }
        fn cmp4ie(w: &mut W<Self>) -> master::dier::CMPIE_W<Self> {
            Self::cmpie(w, 3)
        }
        fn repie(w: &mut W<Self>) -> master::dier::REPIE_W<Self>;
        fn updie(w: &mut W<Self>) -> master::dier::UPDIE_W<Self>;
    }
}

macro_rules! impl_reg {
    ($($r:ident -> &$rty:path;)*) => {
        $(
            #[inline(always)]
            fn $r(&self) -> &$rty {
                self.$r()
            }
        )*
    };
}

macro_rules! impl_read {
    ($($f:ident $(: $n:ident)? -> $fty:path;)*) => {
        $(
            #[inline(always)]
            fn $f(r: &R<Self> $(, $n: u8)?) -> $fty {
                r.$f($($n)?)
            }
        )*
    };
}
macro_rules! impl_write {
    ($($f:ident $(: $n:ident)? -> $fty:path;)*) => {
        $(
            #[inline(always)]
            fn $f(w: &mut W<Self> $(, $n: u8)?) -> $fty {
                w.$f($($n)?)
            }
        )*
    };
}

macro_rules! impl_master_ext {
    ($tim:ident) => {
        impl MasterExt for $tim::RegisterBlock {
            type CRrs = $tim::cr::CRrs;
            type ISRrs = $tim::isr::ISRrs;
            type ICRrs = $tim::icr::ICRrs;
            type DIERrs = $tim::dier::DIERrs;
            impl_reg! {
                isr -> &Reg<Self::ISRrs>;
                icr -> &Reg<Self::ICRrs>;
                dier -> &Reg<Self::DIERrs>;
                cr -> &Reg<Self::CRrs>;
                cntr -> &master::CNTR;
                perr -> &master::PERR;
                repr -> &master::REPR;
                cmp1r -> &master::CMP1R;
                cmp2r -> &master::CMP1R;
                cmp3r -> &master::CMP1R;
                cmp4r -> &master::CMP1R;
            }
        }

        impl reg::MasterCrR for $tim::cr::CRrs {
            impl_read! {
                ckpsc -> master::cr::CKPSC_R;
                cont -> master::cr::CONT_R;
                retrig -> master::cr::RETRIG_R;
                half -> master::cr::HALF_R;
                syncrst -> master::cr::SYNCRST_R;
                syncstrt -> master::cr::SYNCSTRT_R;
                dacsync -> master::cr::DACSYNC_R;
                preen -> master::cr::PREEN_R;
            }
        }
        impl reg::MasterCrW for $tim::cr::CRrs {
            impl_write! {
                ckpsc -> master::cr::CKPSC_W<Self>;
                cont -> master::cr::CONT_W<Self>;
                retrig -> master::cr::RETRIG_W<Self>;
                half -> master::cr::HALF_W<Self>;
                syncrst -> master::cr::SYNCRST_W<Self>;
                syncstrt -> master::cr::SYNCSTRT_W<Self>;
                dacsync -> master::cr::DACSYNC_W<Self>;
                preen -> master::cr::PREEN_W<Self>;
            }
        }
    };
}

impl_master_ext!(master);
impl_master_ext!(tima);
impl_master_ext!(timb);
impl_master_ext!(timc);
impl_master_ext!(timd);
impl_master_ext!(time);
#[cfg(feature = "hrtim_v2")]
impl_master_ext!(timf);

macro_rules! impl_tim_ext {
    ($tim:ident) => {
        impl TimExt for $tim::RegisterBlock {
            type SET1Rrs = $tim::set1r::SET1Rrs;
            type RST1Rrs = $tim::rst1r::RST1Rrs;
            type RSTRrs = $tim::rstr::RSTRrs;
            type CPT1CRrs = $tim::cpt1cr::CPT1CRrs;
            impl_reg! {
                cmp1cr -> &tima::CMP1CR;
                cpt1r -> &tima::CPT1R;
                cpt2r -> &tima::CPT1R;
                dtr -> &tima::DTR;
                eefr1 -> &tima::EEFR1;
                eefr2 -> &tima::EEFR2;
                set1r -> &Reg<Self::SET1Rrs>;
                rst1r -> &Reg<Self::RST1Rrs>;
                set2r -> &Reg<Self::SET1Rrs>;
                rst2r -> &Reg<Self::RST1Rrs>;
                rstr -> &Reg<Self::RSTRrs>;
                chpr -> &tima::CHPR;
                cpt1cr -> &Reg<Self::CPT1CRrs>;
                cpt2cr -> &Reg<Self::CPT1CRrs>;
                outr -> &tima::OUTR;
                fltr -> &tima::FLTR;
            }
            #[cfg(feature = "hrtim_v2")]
            impl_reg! {
                cr2 -> &tima::CR2;
                eefr3 -> &tima::EEFR3;
            }
        }

        impl reg::TimCrR for $tim::cr::CRrs {
            impl_read! {
                pshpll -> tima::cr::PSHPLL_R;
                delcmp2 -> tima::cr::DELCMP2_R;
                delcmp4 -> tima::cr::DELCMP4_R;
                mstu -> tima::cr::MSTU_R;
                trepu -> tima::cr::TREPU_R;
                trstu -> tima::cr::TRSTU_R;
                updgat -> tima::cr::UPDGAT_R;
            }
        }
        impl reg::TimCrW for $tim::cr::CRrs {
            impl_write! {
                pshpll -> tima::cr::PSHPLL_W<Self>;
                delcmp2 -> tima::cr::DELCMP2_W<Self>;
                delcmp4 -> tima::cr::DELCMP4_W<Self>;
                mstu -> tima::cr::MSTU_W<Self>;
                trepu -> tima::cr::TREPU_W<Self>;
                trstu -> tima::cr::TRSTU_W<Self>;
                updgat -> tima::cr::UPDGAT_W<Self>;
            }
        }

        impl reg::CptcrW for $tim::cpt1cr::CPT1CRrs {
            #[inline(always)]
            fn set_swcpt(w: &mut W<Self>) -> &mut W<Self> {
                w.swcpt().set_bit()
            }
        }
    };
}

impl_tim_ext!(tima);
impl_tim_ext!(timb);
impl_tim_ext!(timc);
impl_tim_ext!(timd);
impl_tim_ext!(time);
#[cfg(feature = "hrtim_v2")]
impl_tim_ext!(timf);

macro_rules! impl_irq_ext {
    ($tim:ident) => {
        impl reg::MasterIsr for $tim::isr::ISRrs {
            impl_read! {
                cmp: n -> master::isr::CMP_R;
                rep -> master::isr::REP_R;
                upd -> master::isr::UPD_R;
            }
        }

        impl reg::MasterIcr for $tim::icr::ICRrs {
            impl_write! {
                cmpc: n -> master::icr::CMPC_W<Self>;
                repc -> master::icr::CMPC_W<Self>;
                updc -> master::icr::CMPC_W<Self>;
            }
        }

        impl reg::MasterDierR for $tim::dier::DIERrs {
            impl_read! {
                cmpie: n -> master::dier::CMPIE_R;
                repie -> master::dier::REPIE_R;
                updie -> master::dier::UPDIE_R;
            }
        }
        impl reg::MasterDierW for $tim::dier::DIERrs {
            impl_write! {
                cmpie: n -> master::dier::CMPIE_W<Self>;
                repie -> master::dier::REPIE_W<Self>;
                updie -> master::dier::UPDIE_W<Self>;
            }
        }
    };
}

impl_irq_ext!(master);
impl_irq_ext!(tima);
