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
    type CRrs: MasterCr;
    fn cr(&self) -> &Reg<Self::CRrs>;
    type ISRrs: MasterIsr;
    fn isr(&self) -> &Reg<Self::ISRrs>;
    type ICRrs: MasterIcr;
    fn icr(&self) -> &Reg<Self::ICRrs>;
    type DIERrs: MasterDier;
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
    type CPT1CRrs: Cptcr;
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

pub trait MasterCr: RegisterSpec<Ux = u32> + Readable + Writable + Resettable + Sized {
    fn r_ckpsc(r: &R<Self>) -> master::cr::CKPSC_R;
    fn r_cont(r: &R<Self>) -> master::cr::CONT_R;
    fn r_retrig(r: &R<Self>) -> master::cr::RETRIG_R;
    fn r_half(r: &R<Self>) -> master::cr::HALF_R;
    fn r_syncrst(r: &R<Self>) -> master::cr::SYNCRST_R;
    fn r_syncstrt(r: &R<Self>) -> master::cr::SYNCSTRT_R;
    fn r_dacsync(r: &R<Self>) -> master::cr::DACSYNC_R;
    fn r_preen(r: &R<Self>) -> master::cr::PREEN_R;
    fn w_ckpsc(w: &mut W<Self>) -> master::cr::CKPSC_W<Self>;
    fn w_cont(w: &mut W<Self>) -> master::cr::CONT_W<Self>;
    fn w_retrig(w: &mut W<Self>) -> master::cr::RETRIG_W<Self>;
    fn w_half(w: &mut W<Self>) -> master::cr::HALF_W<Self>;
    fn w_syncrst(w: &mut W<Self>) -> master::cr::SYNCRST_W<Self>;
    fn w_syncstrt(w: &mut W<Self>) -> master::cr::SYNCSTRT_W<Self>;
    fn w_dacsync(w: &mut W<Self>) -> master::cr::DACSYNC_W<Self>;
    fn w_preen(w: &mut W<Self>) -> master::cr::PREEN_W<Self>;
}

pub trait TimCr: MasterCr {
    fn r_pshpll(r: &R<Self>) -> tima::cr::PSHPLL_R;
    fn r_delcmp2(r: &R<Self>) -> tima::cr::DELCMP2_R;
    fn r_delcmp4(r: &R<Self>) -> tima::cr::DELCMP4_R;
    fn r_mstu(r: &R<Self>) -> tima::cr::MSTU_R;
    fn r_trepu(r: &R<Self>) -> tima::cr::TREPU_R;
    fn r_trstu(r: &R<Self>) -> tima::cr::TRSTU_R;
    fn r_updgat(r: &R<Self>) -> tima::cr::UPDGAT_R;
    fn w_pshpll(w: &mut W<Self>) -> tima::cr::PSHPLL_W<Self>;
    fn w_delcmp2(w: &mut W<Self>) -> tima::cr::DELCMP2_W<Self>;
    fn w_delcmp4(w: &mut W<Self>) -> tima::cr::DELCMP4_W<Self>;
    fn w_mstu(w: &mut W<Self>) -> tima::cr::MSTU_W<Self>;
    fn w_trepu(w: &mut W<Self>) -> tima::cr::TREPU_W<Self>;
    fn w_trstu(w: &mut W<Self>) -> tima::cr::TRSTU_W<Self>;
    fn w_updgat(w: &mut W<Self>) -> tima::cr::UPDGAT_W<Self>;
}

pub trait Cptcr: RegisterSpec<Ux = u32> + Readable + Writable + Resettable + Sized {
    // TODO: replace this
    fn set_swcpt(w: &mut W<Self>) -> &mut W<Self>;
}

macro_rules! impl_master_ext {
    ($tim:ident) => {
        impl MasterExt for $tim::RegisterBlock {
            type CRrs = $tim::cr::CRrs;
            type ISRrs = $tim::isr::ISRrs;
            type ICRrs = $tim::icr::ICRrs;
            type DIERrs = $tim::dier::DIERrs;
            fn isr(&self) -> &Reg<Self::ISRrs> {
                self.isr()
            }
            fn icr(&self) -> &Reg<Self::ICRrs> {
                self.icr()
            }
            fn dier(&self) -> &Reg<Self::DIERrs> {
                self.dier()
            }
            fn cr(&self) -> &Reg<Self::CRrs> {
                self.cr()
            }
            fn cntr(&self) -> &master::CNTR {
                self.cntr()
            }
            fn perr(&self) -> &master::PERR {
                self.perr()
            }
            fn repr(&self) -> &master::REPR {
                self.repr()
            }
            fn cmp1r(&self) -> &master::CMP1R {
                self.cmp1r()
            }
            fn cmp2r(&self) -> &master::CMP1R {
                self.cmp2r()
            }
            fn cmp3r(&self) -> &master::CMP1R {
                self.cmp3r()
            }
            fn cmp4r(&self) -> &master::CMP1R {
                self.cmp4r()
            }
        }

        impl MasterCr for $tim::cr::CRrs {
            fn r_ckpsc(r: &R<Self>) -> master::cr::CKPSC_R {
                r.ckpsc()
            }
            fn r_cont(r: &R<Self>) -> master::cr::CONT_R {
                r.cont()
            }
            fn r_retrig(r: &R<Self>) -> master::cr::RETRIG_R {
                r.retrig()
            }
            fn r_half(r: &R<Self>) -> master::cr::HALF_R {
                r.half()
            }
            fn r_syncrst(r: &R<Self>) -> master::cr::SYNCRST_R {
                r.syncrst()
            }
            fn r_syncstrt(r: &R<Self>) -> master::cr::SYNCSTRT_R {
                r.syncstrt()
            }
            fn r_dacsync(r: &R<Self>) -> master::cr::DACSYNC_R {
                r.dacsync()
            }
            fn r_preen(r: &R<Self>) -> master::cr::PREEN_R {
                r.preen()
            }
            fn w_ckpsc(w: &mut W<Self>) -> master::cr::CKPSC_W<Self> {
                w.ckpsc()
            }
            fn w_cont(w: &mut W<Self>) -> master::cr::CONT_W<Self> {
                w.cont()
            }
            fn w_retrig(w: &mut W<Self>) -> master::cr::RETRIG_W<Self> {
                w.retrig()
            }
            fn w_half(w: &mut W<Self>) -> master::cr::HALF_W<Self> {
                w.half()
            }
            fn w_syncrst(w: &mut W<Self>) -> master::cr::SYNCRST_W<Self> {
                w.syncrst()
            }
            fn w_syncstrt(w: &mut W<Self>) -> master::cr::SYNCSTRT_W<Self> {
                w.syncstrt()
            }
            fn w_dacsync(w: &mut W<Self>) -> master::cr::DACSYNC_W<Self> {
                w.dacsync()
            }
            fn w_preen(w: &mut W<Self>) -> master::cr::PREEN_W<Self> {
                w.preen()
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
            fn cmp1cr(&self) -> &tima::CMP1CR {
                self.cmp1cr()
            }
            fn cpt1r(&self) -> &tima::CPT1R {
                self.cpt1r()
            }
            fn cpt2r(&self) -> &tima::CPT1R {
                self.cpt2r()
            }
            fn dtr(&self) -> &tima::DTR {
                self.dtr()
            }
            fn eefr1(&self) -> &tima::EEFR1 {
                self.eefr1()
            }
            fn eefr2(&self) -> &tima::EEFR2 {
                self.eefr2()
            }
            type SET1Rrs = $tim::set1r::SET1Rrs;
            type RST1Rrs = $tim::rst1r::RST1Rrs;
            fn set1r(&self) -> &Reg<Self::SET1Rrs> {
                self.set1r()
            }
            fn rst1r(&self) -> &Reg<Self::RST1Rrs> {
                self.rst1r()
            }
            fn set2r(&self) -> &Reg<Self::SET1Rrs> {
                self.set2r()
            }
            fn rst2r(&self) -> &Reg<Self::RST1Rrs> {
                self.rst1r()
            }
            type RSTRrs = $tim::rstr::RSTRrs;
            fn rstr(&self) -> &Reg<Self::RSTRrs> {
                self.rstr()
            }
            fn chpr(&self) -> &tima::CHPR {
                self.chpr()
            }
            type CPT1CRrs = $tim::cpt1cr::CPT1CRrs;
            fn cpt1cr(&self) -> &Reg<Self::CPT1CRrs> {
                self.cpt1cr()
            }
            fn cpt2cr(&self) -> &Reg<Self::CPT1CRrs> {
                self.cpt2cr()
            }
            fn outr(&self) -> &tima::OUTR {
                self.outr()
            }
            fn fltr(&self) -> &tima::FLTR {
                self.fltr()
            }
            #[cfg(feature = "hrtim_v2")]
            fn cr2(&self) -> &tima::CR2 {
                self.cr2()
            }
            #[cfg(feature = "hrtim_v2")]
            fn eefr3(&self) -> &tima::EEFR3 {
                self.eefr3()
            }
        }

        impl TimCr for $tim::cr::CRrs {
            fn r_pshpll(r: &R<Self>) -> tima::cr::PSHPLL_R {
                r.pshpll()
            }
            fn r_delcmp2(r: &R<Self>) -> tima::cr::DELCMP2_R {
                r.delcmp2()
            }
            fn r_delcmp4(r: &R<Self>) -> tima::cr::DELCMP4_R {
                r.delcmp4()
            }
            fn r_mstu(r: &R<Self>) -> tima::cr::MSTU_R {
                r.mstu()
            }
            fn r_trepu(r: &R<Self>) -> tima::cr::TREPU_R {
                r.trepu()
            }
            fn r_trstu(r: &R<Self>) -> tima::cr::TRSTU_R {
                r.trstu()
            }
            fn r_updgat(r: &R<Self>) -> tima::cr::UPDGAT_R {
                r.updgat()
            }
            fn w_pshpll(w: &mut W<Self>) -> tima::cr::PSHPLL_W<Self> {
                w.pshpll()
            }
            fn w_delcmp2(w: &mut W<Self>) -> tima::cr::DELCMP2_W<Self> {
                w.delcmp2()
            }
            fn w_delcmp4(w: &mut W<Self>) -> tima::cr::DELCMP4_W<Self> {
                w.delcmp4()
            }
            fn w_mstu(w: &mut W<Self>) -> tima::cr::MSTU_W<Self> {
                w.mstu()
            }
            fn w_trepu(w: &mut W<Self>) -> tima::cr::TREPU_W<Self> {
                w.trepu()
            }
            fn w_trstu(w: &mut W<Self>) -> tima::cr::TRSTU_W<Self> {
                w.trstu()
            }
            fn w_updgat(w: &mut W<Self>) -> tima::cr::UPDGAT_W<Self> {
                w.updgat()
            }
        }

        impl Cptcr for $tim::cpt1cr::CPT1CRrs {
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
pub trait MasterDier: RegisterSpec<Ux = u32> + Readable + Writable + Resettable + Sized {
    fn r_cmpie(r: &R<Self>, n: u8) -> master::dier::CMPIE_R;
    fn r_cmp1ie(r: &R<Self>) -> master::dier::CMPIE_R {
        Self::r_cmpie(r, 0)
    }
    fn r_cmp2ie(r: &R<Self>) -> master::dier::CMPIE_R {
        Self::r_cmpie(r, 1)
    }
    fn r_cmp3ie(r: &R<Self>) -> master::dier::CMPIE_R {
        Self::r_cmpie(r, 2)
    }
    fn r_cmp4ie(r: &R<Self>) -> master::dier::CMPIE_R {
        Self::r_cmpie(r, 3)
    }
    fn r_repie(r: &R<Self>) -> master::dier::REPIE_R;
    fn r_updie(r: &R<Self>) -> master::dier::UPDIE_R;

    fn w_cmpie(w: &mut W<Self>, n: u8) -> master::dier::CMPIE_W<Self>;
    fn w_cmp1ie(w: &mut W<Self>) -> master::dier::CMPIE_W<Self> {
        Self::w_cmpie(w, 0)
    }
    fn w_cmp2ie(w: &mut W<Self>) -> master::dier::CMPIE_W<Self> {
        Self::w_cmpie(w, 1)
    }
    fn w_cmp3ie(w: &mut W<Self>) -> master::dier::CMPIE_W<Self> {
        Self::w_cmpie(w, 2)
    }
    fn w_cmp4ie(w: &mut W<Self>) -> master::dier::CMPIE_W<Self> {
        Self::w_cmpie(w, 3)
    }
    fn w_repie(w: &mut W<Self>) -> master::dier::REPIE_W<Self>;
    fn w_updie(w: &mut W<Self>) -> master::dier::UPDIE_W<Self>;
}

macro_rules! impl_irq_ext {
    ($tim:ident) => {
        impl MasterIsr for $tim::isr::ISRrs {
            fn cmp(r: &R<Self>, n: u8) -> master::isr::CMP_R {
                r.cmp(n)
            }
            fn rep(r: &R<Self>) -> master::isr::REP_R {
                r.rep()
            }
            fn upd(r: &R<Self>) -> master::isr::UPD_R {
                r.upd()
            }
        }

        impl MasterIcr for $tim::icr::ICRrs {
            fn cmpc(w: &mut W<Self>, n: u8) -> master::icr::CMPC_W<Self> {
                w.cmpc(n)
            }
            fn repc(w: &mut W<Self>) -> master::icr::CMPC_W<Self> {
                w.repc()
            }
            fn updc(w: &mut W<Self>) -> master::icr::CMPC_W<Self> {
                w.updc()
            }
        }

        impl MasterDier for $tim::dier::DIERrs {
            fn r_cmpie(r: &R<Self>, n: u8) -> master::dier::CMPIE_R {
                r.cmpie(n)
            }
            fn r_repie(r: &R<Self>) -> master::dier::REPIE_R {
                r.repie()
            }
            fn r_updie(r: &R<Self>) -> master::dier::UPDIE_R {
                r.updie()
            }
            fn w_cmpie(w: &mut W<Self>, n: u8) -> master::dier::CMPIE_W<Self> {
                w.cmpie(n)
            }
            fn w_repie(w: &mut W<Self>) -> master::dier::REPIE_W<Self> {
                w.repie()
            }
            fn w_updie(w: &mut W<Self>) -> master::dier::UPDIE_W<Self> {
                w.updie()
            }
        }
    };
}

impl_irq_ext!(master);
impl_irq_ext!(tima);
