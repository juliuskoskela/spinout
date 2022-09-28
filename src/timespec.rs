use std::fmt;
use std::time::Duration;

pub use self::inner::Instant;

const NSEC_PER_SEC: u64 = 1_000_000_000;
// pub const UNIX_EPOCH: SystemTime = SystemTime { t: Timespec::zero() };

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime {
    pub t: Timespec,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

// impl SystemTime {
//     #[cfg_attr(target_os = "horizon", allow(unused))]
//     pub fn new(tv_sec: i64, tv_nsec: i64) -> SystemTime {
//         SystemTime { t: Timespec::new(tv_sec, tv_nsec) }
//     }

//     pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
//         self.t.sub_timespec(&other.t)
//     }

//     pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
//         Some(SystemTime { t: self.t.checked_add_duration(other)? })
//     }

//     pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
//         Some(SystemTime { t: self.t.checked_sub_duration(other)? })
//     }
// }

impl From<libc::timespec> for SystemTime {
    fn from(t: libc::timespec) -> SystemTime {
        SystemTime { t: Timespec::from(t) }
    }
}

impl fmt::Debug for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemTime")
            .field("tv_sec", &self.t.tv_sec)
            .field("tv_nsec", &self.t.tv_nsec)
            .finish()
    }
}

impl Timespec {
    fn new(tv_sec: i64, tv_nsec: i64) -> Timespec {
        Timespec { tv_sec, tv_nsec }
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Timespec> {
        let mut secs = other
            .as_secs()
            .try_into() // <- target type would be `i64`
            .ok()
            .and_then(|secs| self.tv_sec.checked_add(secs))?;

        // Nano calculations can't overflow because nanos are <1B which fit
        // in a u32.
        let mut nsec = other.subsec_nanos() + self.tv_nsec as u32;
        if nsec >= NSEC_PER_SEC as u32 {
            nsec -= NSEC_PER_SEC as u32;
            secs = secs.checked_add(1)?;
        }
        Some(Timespec::new(secs, nsec as i64))
    }

    #[allow(dead_code)]
    pub fn to_timespec(&self) -> Option<libc::timespec> {
        Some(libc::timespec {
            tv_sec: self.tv_sec.try_into().ok()?,
            tv_nsec: self.tv_nsec.try_into().ok()?,
        })
    }
}

impl From<libc::timespec> for Timespec {
    fn from(t: libc::timespec) -> Timespec {
        Timespec::new(t.tv_sec as i64, t.tv_nsec as i64)
    }
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "watchos"))]
mod inner {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sys::cvt;
    use std::sys_common::mul_div_u64;
    use std::time::Duration;

    use super::{SystemTime, Timespec, NSEC_PER_SEC};

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
    pub struct Instant {
        t: u64,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct mach_timebase_info {
        numer: u32,
        denom: u32,
    }
    type mach_timebase_info_t = *mut mach_timebase_info;
    type kern_return_t = libc::c_int;

    impl Instant {
        pub fn now() -> Instant {
            extern "C" {
                fn mach_absolute_time() -> u64;
            }
            Instant { t: unsafe { mach_absolute_time() } }
        }

        pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
            let diff = self.t.checked_sub(other.t)?;
            let info = info();
            let nanos = mul_div_u64(diff, info.numer as u64, info.denom as u64);
            Some(Duration::new(nanos / NSEC_PER_SEC, (nanos % NSEC_PER_SEC) as u32))
        }

        pub fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
            Some(Instant { t: self.t.checked_add(checked_dur2intervals(other)?)? })
        }

        pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
            Some(Instant { t: self.t.checked_sub(checked_dur2intervals(other)?)? })
        }
    }

    impl SystemTime {
        pub fn now() -> SystemTime {
            use std::ptr;

            let mut s = libc::timeval { tv_sec: 0, tv_usec: 0 };
            cvt(unsafe { libc::gettimeofday(&mut s, ptr::null_mut()) }).unwrap();
            return SystemTime::from(s);
        }
    }

    impl From<libc::timeval> for Timespec {
        fn from(t: libc::timeval) -> Timespec {
            Timespec::new(t.tv_sec as i64, 1000 * t.tv_usec as i64)
        }
    }

    impl From<libc::timeval> for SystemTime {
        fn from(t: libc::timeval) -> SystemTime {
            SystemTime { t: Timespec::from(t) }
        }
    }

    fn checked_dur2intervals(dur: &Duration) -> Option<u64> {
        let nanos =
            dur.as_secs().checked_mul(NSEC_PER_SEC)?.checked_add(dur.subsec_nanos() as u64)?;
        let info = info();
        Some(mul_div_u64(nanos, info.denom as u64, info.numer as u64))
    }

    fn info() -> mach_timebase_info {
        // INFO_BITS conceptually is an `Option<mach_timebase_info>`. We can do
        // this in 64 bits because we know 0 is never a valid value for the
        // `denom` field.
        //
        // Encoding this as a single `AtomicU64` allows us to use `Relaxed`
        // operations, as we are only interested in the effects on a single
        // memory location.
        static INFO_BITS: AtomicU64 = AtomicU64::new(0);

        // If a previous thread has initialized `INFO_BITS`, use it.
        let info_bits = INFO_BITS.load(Ordering::Relaxed);
        if info_bits != 0 {
            return info_from_bits(info_bits);
        }

        // ... otherwise learn for ourselves ...
        extern "C" {
            fn mach_timebase_info(info: mach_timebase_info_t) -> kern_return_t;
        }

        let mut info = info_from_bits(0);
        unsafe {
            mach_timebase_info(&mut info);
        }
        INFO_BITS.store(info_to_bits(info), Ordering::Relaxed);
        info
    }

    #[inline]
    fn info_to_bits(info: mach_timebase_info) -> u64 {
        ((info.denom as u64) << 32) | (info.numer as u64)
    }

    #[inline]
    fn info_from_bits(bits: u64) -> mach_timebase_info {
        mach_timebase_info { numer: bits as u32, denom: (bits >> 32) as u32 }
    }
}

pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

pub fn cvt<T: IsMinusOne>(t: T) -> std::io::Result<T> {
    if t.is_minus_one() { Err(std::io::Error::last_os_error()) } else { Ok(t) }
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "watchos")))]
mod inner {
    use std::fmt;
    use std::mem::MaybeUninit;
    use crate::timespec::cvt;

    use super::Timespec;

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant {
        t: Timespec,
    }

    impl fmt::Debug for Instant {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Instant")
                .field("tv_sec", &self.t.tv_sec)
                .field("tv_nsec", &self.t.tv_nsec)
                .finish()
        }
    }

    #[cfg(not(any(target_os = "dragonfly", target_os = "espidf", target_os = "horizon")))]
	#[allow(non_camel_case_types)]
    pub type clock_t = libc::c_int;
    #[cfg(any(target_os = "dragonfly", target_os = "espidf", target_os = "horizon"))]
    pub type clock_t = libc::c_ulong;

    impl Timespec {
        pub fn now(clock: clock_t) -> Timespec {
            // Try to use 64-bit time in preparation for Y2038.
            #[cfg(all(target_os = "linux", target_env = "gnu", target_pointer_width = "32"))]
            {
                use std::sys::weak::weak;

                // __clock_gettime64 was added to 32-bit arches in glibc 2.34,
                // and it handles both vDSO calls and ENOSYS fallbacks itself.
                weak!(fn __clock_gettime64(libc::clockid_t, *mut __timespec64) -> libc::c_int);

                #[repr(C)]
                struct __timespec64 {
                    tv_sec: i64,
                    #[cfg(target_endian = "big")]
                    _padding: i32,
                    tv_nsec: i32,
                    #[cfg(target_endian = "little")]
                    _padding: i32,
                }

                if let Some(clock_gettime64) = __clock_gettime64.get() {
                    let mut t = MaybeUninit::uninit();
                    cvt(unsafe { clock_gettime64(clock, t.as_mut_ptr()) }).unwrap();
                    let t = unsafe { t.assume_init() };
                    return Timespec { tv_sec: t.tv_sec, tv_nsec: t.tv_nsec as i64 };
                }
            }

            let mut t = MaybeUninit::uninit();
            cvt(unsafe { libc::clock_gettime(clock, t.as_mut_ptr()) }).unwrap();
            Timespec::from(unsafe { t.assume_init() })
        }
    }
}

